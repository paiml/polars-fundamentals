// serve — Module 4: Serving the Gold Layer
//
// An Axum HTTP API over the gold LazyFrame. Every endpoint is a Polars
// expression evaluated against the gold DataFrame held in memory:
//
//   GET /wines                   filter() on region/variety/min_rating/max_rating
//   GET /regions                 group_by("region") with a count
//   GET /varieties               group_by("variety") with count and mean rating
//   GET /wines/search?q=         substring match on name or notes
//   GET /wines/region/:region    exact region match
//
// Query parameters never become strings of code: each one is passed to lit(),
// so a region like "Napa' OR 1=1" is just a region nobody has.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Rows returned by a list endpoint when the caller does not ask for a limit.
pub const DEFAULT_LIMIT: u32 = 100;
/// The most rows any single request may return.
pub const MAX_LIMIT: u32 = 1000;

#[derive(Clone)]
pub struct AppState {
    gold: Arc<DataFrame>,
}

impl AppState {
    pub fn new(gold: DataFrame) -> Self {
        Self {
            gold: Arc::new(gold),
        }
    }

    /// A fresh LazyFrame over the gold data. Cloning a DataFrame clones
    /// reference-counted column handles, not the data itself.
    fn lazy(&self) -> LazyFrame {
        self.gold.as_ref().clone().lazy()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Wine {
    pub name: String,
    pub variety: String,
    pub region: Option<String>,
    pub rating: f64,
    pub notes: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct WineFilters {
    pub region: Option<String>,
    pub variety: Option<String>,
    pub min_rating: Option<f64>,
    pub max_rating: Option<f64>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct LimitQuery {
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct VarietyInfo {
    pub count: u32,
    pub avg_rating: f64,
}

/// A Polars error becomes a 500 with the message as JSON, never a panic.
pub struct ApiError(PolarsError);

impl From<PolarsError> for ApiError {
    fn from(e: PolarsError) -> Self {
        Self(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({ "error": self.0.to_string() }));
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

/// Case-insensitive substring match: the Polars form of SQL's `LIKE '%q%'`.
fn contains_ci(column: &str, needle: &str) -> Expr {
    col(column)
        .str()
        .to_lowercase()
        .str()
        .contains_literal(lit(needle.to_lowercase()))
}

fn clamp_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT)
}

/// Collect a LazyFrame of gold rows into the JSON shape, highest rating first.
fn to_wines(lf: LazyFrame, limit: u32) -> PolarsResult<Vec<Wine>> {
    let df = lf
        .sort(
            ["rating", "name"],
            SortMultipleOptions::default().with_order_descending_multi([true, false]),
        )
        .limit(limit)
        .collect()?;

    let name = df.column("name")?.str()?;
    let variety = df.column("variety")?.str()?;
    let region = df.column("region")?.str()?;
    let rating = df.column("rating")?.f64()?;
    let notes = df.column("notes")?.str()?;

    Ok((0..df.height())
        .map(|i| Wine {
            name: name.get(i).unwrap_or_default().to_string(),
            variety: variety.get(i).unwrap_or_default().to_string(),
            region: region.get(i).map(str::to_string),
            rating: rating.get(i).unwrap_or_default(),
            notes: notes.get(i).unwrap_or_default().to_string(),
        })
        .collect())
}

/// Build the filter for GET /wines. Each supplied parameter adds one
/// predicate; with none supplied the expression keeps every row.
pub fn wine_filter(f: &WineFilters) -> Expr {
    let mut predicate = lit(true);
    if let Some(region) = &f.region {
        predicate = predicate.and(contains_ci("region", region));
    }
    if let Some(variety) = &f.variety {
        predicate = predicate.and(contains_ci("variety", variety));
    }
    if let Some(min) = f.min_rating {
        predicate = predicate.and(col("rating").gt_eq(lit(min)));
    }
    if let Some(max) = f.max_rating {
        predicate = predicate.and(col("rating").lt_eq(lit(max)));
    }
    predicate
}

async fn get_wines(
    State(state): State<AppState>,
    Query(filters): Query<WineFilters>,
) -> ApiResult<Vec<Wine>> {
    let lf = state.lazy().filter(wine_filter(&filters));
    Ok(Json(to_wines(lf, clamp_limit(filters.limit))?))
}

async fn get_regions(State(state): State<AppState>) -> ApiResult<BTreeMap<String, u32>> {
    let df = state
        .lazy()
        .filter(col("region").is_not_null())
        .group_by([col("region")])
        .agg([len().alias("count")])
        .collect()?;

    let region = df.column("region")?.str()?;
    let count = df.column("count")?.u32()?;
    Ok(Json(
        (0..df.height())
            .filter_map(|i| Some((region.get(i)?.to_string(), count.get(i)?)))
            .collect(),
    ))
}

async fn get_varieties(State(state): State<AppState>) -> ApiResult<BTreeMap<String, VarietyInfo>> {
    let df = state
        .lazy()
        .group_by([col("variety")])
        .agg([
            len().alias("count"),
            col("rating").mean().alias("avg_rating"),
        ])
        .collect()?;

    let variety = df.column("variety")?.str()?;
    let count = df.column("count")?.u32()?;
    let avg = df.column("avg_rating")?.f64()?;
    Ok(Json(
        (0..df.height())
            .filter_map(|i| {
                Some((
                    variety.get(i)?.to_string(),
                    VarietyInfo {
                        count: count.get(i)?,
                        avg_rating: avg.get(i)?,
                    },
                ))
            })
            .collect(),
    ))
}

async fn search_wines(
    State(state): State<AppState>,
    Query(search): Query<SearchQuery>,
) -> ApiResult<Vec<Wine>> {
    let lf = state
        .lazy()
        .filter(contains_ci("name", &search.q).or(contains_ci("notes", &search.q)));
    Ok(Json(to_wines(lf, clamp_limit(search.limit))?))
}

async fn get_wines_by_region(
    State(state): State<AppState>,
    Path(region): Path<String>,
    Query(q): Query<LimitQuery>,
) -> ApiResult<Vec<Wine>> {
    let lf = state.lazy().filter(col("region").eq(lit(region)));
    Ok(Json(to_wines(lf, clamp_limit(q.limit))?))
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/wines/search", get(search_wines))
        .route("/wines/region/:region", get(get_wines_by_region))
        .route("/wines", get(get_wines))
        .route("/regions", get(get_regions))
        .route("/varieties", get(get_varieties))
        .with_state(state)
}

/// Bind and serve until the process is stopped.
pub async fn run(gold: DataFrame, addr: std::net::SocketAddr) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Serving {} gold wines on http://{addr}", gold.height());
    axum::serve(listener, create_app(AppState::new(gold))).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    /// Five gold rows, one with no region: small enough to check by hand.
    fn fixture() -> DataFrame {
        df![
            "name" => ["Achaval Ferrer Malbec", "Catena Zapata Malbec", "Louis Jadot Beaune", "Elk Cove Pinot", "Mystery Blend"],
            "variety" => ["RED WINE", "RED WINE", "RED WINE", "RED WINE", "WHITE WINE"],
            "region" => [Some("Mendoza, Argentina"), Some("Mendoza, Argentina"), Some("Burgundy, France"), Some("Oregon"), None],
            "rating" => [94.0, 91.0, 92.0, 90.0, 93.0],
            "notes" => ["Dark fruit, tobacco", "Violet and plum", "Cherry, earthy", "Red berry, silky", "Honeyed citrus"],
        ]
        .expect("fixture frame")
    }

    async fn get_json<T: serde::de::DeserializeOwned>(uri: &str) -> (StatusCode, T) {
        let app = create_app(AppState::new(fixture()));
        let res = app
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).expect("JSON body"))
    }

    fn names(wines: &[Wine]) -> Vec<&str> {
        wines.iter().map(|w| w.name.as_str()).collect()
    }

    #[tokio::test]
    async fn wines_without_filters_returns_every_row_highest_rating_first() {
        let (status, wines): (_, Vec<Wine>) = get_json("/wines").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(wines.len(), 5);
        let ratings: Vec<f64> = wines.iter().map(|w| w.rating).collect();
        assert_eq!(ratings, vec![94.0, 93.0, 92.0, 91.0, 90.0]);
    }

    #[tokio::test]
    async fn wines_region_filter_is_a_case_insensitive_substring() {
        let (_, wines): (_, Vec<Wine>) = get_json("/wines?region=mendoza").await;
        assert_eq!(
            names(&wines),
            vec!["Achaval Ferrer Malbec", "Catena Zapata Malbec"]
        );
    }

    #[tokio::test]
    async fn wines_rating_bounds_are_inclusive_and_combine_with_and() {
        let (_, wines): (_, Vec<Wine>) =
            get_json("/wines?variety=red&min_rating=91&max_rating=92").await;
        assert_eq!(
            names(&wines),
            vec!["Louis Jadot Beaune", "Catena Zapata Malbec"]
        );
    }

    #[tokio::test]
    async fn wines_min_rating_above_every_rating_is_an_empty_list_not_an_error() {
        let (status, wines): (_, Vec<Wine>) = get_json("/wines?min_rating=99.5").await;
        assert_eq!(status, StatusCode::OK);
        assert!(wines.is_empty());
    }

    #[tokio::test]
    async fn wines_limit_truncates_after_sorting() {
        let (_, wines): (_, Vec<Wine>) = get_json("/wines?limit=2").await;
        assert_eq!(
            names(&wines),
            vec!["Achaval Ferrer Malbec", "Mystery Blend"]
        );
    }

    #[tokio::test]
    async fn wines_region_value_is_data_not_code() {
        let (_, wines): (_, Vec<Wine>) = get_json("/wines?region=x%27%20OR%201%3D1").await;
        assert!(wines.is_empty());
    }

    #[tokio::test]
    async fn regions_counts_every_region_and_skips_missing_ones() {
        let (_, regions): (_, BTreeMap<String, u32>) = get_json("/regions").await;
        let expected: BTreeMap<String, u32> = [
            ("Burgundy, France".to_string(), 1),
            ("Mendoza, Argentina".to_string(), 2),
            ("Oregon".to_string(), 1),
        ]
        .into();
        assert_eq!(regions, expected);
        assert_eq!(regions.values().sum::<u32>(), 4);
    }

    #[tokio::test]
    async fn varieties_report_count_and_mean_rating() {
        let (_, varieties): (_, BTreeMap<String, VarietyInfo>) = get_json("/varieties").await;
        assert_eq!(varieties.len(), 2);
        let red = &varieties["RED WINE"];
        assert_eq!(red.count, 4);
        assert!((red.avg_rating - 91.75).abs() < 1e-9);
        assert_eq!(
            varieties["WHITE WINE"],
            VarietyInfo {
                count: 1,
                avg_rating: 93.0
            }
        );
    }

    #[tokio::test]
    async fn search_matches_name_or_notes() {
        let (_, by_name): (_, Vec<Wine>) = get_json("/wines/search?q=elk").await;
        assert_eq!(names(&by_name), vec!["Elk Cove Pinot"]);
        let (_, by_notes): (_, Vec<Wine>) = get_json("/wines/search?q=TOBACCO").await;
        assert_eq!(names(&by_notes), vec!["Achaval Ferrer Malbec"]);
    }

    #[tokio::test]
    async fn search_without_q_is_rejected() {
        let app = create_app(AppState::new(fixture()));
        let res = app
            .oneshot(Request::get("/wines/search").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn region_path_is_an_exact_match() {
        let (_, wines): (_, Vec<Wine>) = get_json("/wines/region/Oregon").await;
        assert_eq!(names(&wines), vec!["Elk Cove Pinot"]);
        let (_, partial): (_, Vec<Wine>) = get_json("/wines/region/Mendoza").await;
        assert!(
            partial.is_empty(),
            "a substring must not match the path route"
        );
    }

    #[tokio::test]
    async fn region_path_decodes_spaces_and_commas() {
        let (_, wines): (_, Vec<Wine>) = get_json("/wines/region/Mendoza%2C%20Argentina").await;
        assert_eq!(wines.len(), 2);
    }

    #[tokio::test]
    async fn nonexistent_region_is_an_empty_list() {
        let (status, wines): (_, Vec<Wine>) = get_json("/wines/region/Atlantis").await;
        assert_eq!(status, StatusCode::OK);
        assert!(wines.is_empty());
    }

    #[test]
    fn limit_is_clamped() {
        assert_eq!(clamp_limit(None), DEFAULT_LIMIT);
        assert_eq!(clamp_limit(Some(5)), 5);
        assert_eq!(clamp_limit(Some(1_000_000)), MAX_LIMIT);
    }
}
