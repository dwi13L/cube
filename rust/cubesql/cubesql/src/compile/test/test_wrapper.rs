use datafusion::physical_plan::displayable;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::sync::Arc;

use crate::{
    compile::{
        rewrite::rewriter::Rewriter,
        test::{
            convert_select_to_query_plan, convert_select_to_query_plan_customized,
            convert_select_to_query_plan_with_config, init_testing_logger, LogicalPlanTestUtils,
        },
        DatabaseProtocol,
    },
    config::ConfigObjImpl,
    transport::TransportLoadRequestQuery,
};

#[tokio::test]
async fn test_simple_wrapper() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT COALESCE(customer_gender, 'N/A', 'NN'), AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("COALESCE"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_wrapper_group_by_rollup() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT customer_gender, notes, AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1, ROLLUP(2)"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("Rollup"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_wrapper_group_by_rollup_with_aliases() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT customer_gender as \"customer_gender1\", notes as \"notes\", AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY ROLLUP(1, 2)"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("Rollup"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_wrapper_group_by_rollup_nested() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT customer_gender, notes, avg(mp) from (SELECT customer_gender, notes, avg(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1, 2) b GROUP BY ROLLUP(1, 2)"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("ROLLUP(1, 2)"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_wrapper_group_by_rollup_nested_from_asterisk() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT customer_gender, notes, avg(avgPrice) from (SELECT * FROM KibanaSampleDataEcommerce) b GROUP BY ROLLUP(1, 2) ORDER BY 1"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("Rollup"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_wrapper_group_by_rollup_nested_with_aliases() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT customer_gender as \"gender\", notes as \"notes\", avg(mp) from (SELECT customer_gender, notes, avg(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1, 2) b GROUP BY ROLLUP(1, 2)"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("ROLLUP(1, 2)"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_wrapper_group_by_rollup_nested_complex() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT customer_gender, notes, order_date, last_mod, avg(mp) from \
            (SELECT customer_gender, notes, order_date, last_mod, avg(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1, 2, 3, 4) b \
            GROUP BY ROLLUP(1), ROLLUP(2), 3, CUBE(4)"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("ROLLUP(1), ROLLUP(2), 3, CUBE(4)"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_wrapper_group_by_rollup_placeholders() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT customer_gender, notes, AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY ROLLUP(1, 2)"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("Rollup"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_wrapper_group_by_cube() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT customer_gender, notes, AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY CUBE(customer_gender, notes)"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("Cube"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_wrapper_group_by_rollup_complex() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT customer_gender, notes, has_subscription, AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY ROLLUP(customer_gender, notes), has_subscription"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("Rollup"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_simple_subquery_wrapper_projection_empty_source() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT (SELECT 'male' where 1  group by 'male' having 1 order by 'male' limit 1) as gender, avgPrice FROM KibanaSampleDataEcommerce a"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("(SELECT"));
    assert!(sql.contains("utf8__male__"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
    //println!("phys plan {:?}", physical_plan);
}

#[tokio::test]
async fn test_simple_subquery_wrapper_filter_empty_source() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT avgPrice FROM KibanaSampleDataEcommerce a where customer_gender = (SELECT 'male' )"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
    .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("(SELECT"));
    assert!(sql.contains("utf8__male__"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
    //println!("phys plan {:?}", physical_plan);
}

#[tokio::test]
async fn test_simple_subquery_wrapper_projection_aggregate_empty_source() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT (SELECT 'male'), avg(avgPrice) FROM KibanaSampleDataEcommerce a GROUP BY 1"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
    .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("(SELECT"));
    assert!(sql.contains("utf8__male__"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_simple_subquery_wrapper_filter_in_empty_source() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT customer_gender, avgPrice FROM KibanaSampleDataEcommerce a where customer_gender in (select 'male')"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("IN (SELECT"));
    assert!(sql.contains("utf8__male__"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_simple_subquery_wrapper_filter_and_projection_empty_source() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT (select 'male'), avgPrice FROM KibanaSampleDataEcommerce a where customer_gender in (select 'female')"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();

    let sql = logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql;
    assert!(sql.contains("IN (SELECT"));
    assert!(sql.contains("(SELECT"));
    assert!(sql.contains("utf8__male__"));
    assert!(sql.contains("utf8__female__"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_simple_subquery_wrapper_projection() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT (SELECT customer_gender FROM KibanaSampleDataEcommerce LIMIT 1) as gender, avgPrice FROM KibanaSampleDataEcommerce a"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("(SELECT"));
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("\\\\\\\"limit\\\\\\\":1"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_simple_subquery_wrapper_projection_aggregate() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT (SELECT customer_gender FROM KibanaSampleDataEcommerce WHERE customer_gender = 'male' LIMIT 1), avg(avgPrice) FROM KibanaSampleDataEcommerce a GROUP BY 1"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("(SELECT"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_simple_subquery_wrapper_filter_equal() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT customer_gender, avgPrice FROM KibanaSampleDataEcommerce a where customer_gender = (select customer_gender from KibanaSampleDataEcommerce limit 1)"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("(SELECT"));
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("\\\\\\\"limit\\\\\\\":1"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_simple_subquery_wrapper_filter_in() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT customer_gender, avgPrice FROM KibanaSampleDataEcommerce a where customer_gender in (select customer_gender from KibanaSampleDataEcommerce)"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("IN (SELECT"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_simple_subquery_wrapper_filter_and_projection() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT (select customer_gender from KibanaSampleDataEcommerce limit 1), avgPrice FROM KibanaSampleDataEcommerce a where customer_gender in (select customer_gender from KibanaSampleDataEcommerce)"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();

    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("IN (SELECT"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

#[tokio::test]
async fn test_case_wrapper() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT CASE WHEN COALESCE(customer_gender, 'N/A', 'NN') = 'female' THEN 'f' ELSE 'm' END, AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("CASE WHEN"));

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );
}

#[tokio::test]
async fn test_case_wrapper_distinct() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        r#"SELECT CASE WHEN customer_gender = 'female' THEN 'f' ELSE 'm' END, COUNT(DISTINCT countDistinct) mp
            FROM KibanaSampleDataEcommerce a
            WHERE
              (
                (
                  ( a.order_date ) >= '2024-01-01'
                  AND ( a.order_date ) < '2024-02-01'
                )
              )
            GROUP BY 1"#
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("CASE WHEN"));

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );
}

#[tokio::test]
async fn test_case_wrapper_alias_with_order() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT CASE WHEN customer_gender = 'female' THEN 'f' ELSE 'm' END AS \"f822c516-3515-11c2-8464-5d4845a02f73\", AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY CASE WHEN customer_gender = 'female' THEN 'f' ELSE 'm' END ORDER BY CASE WHEN customer_gender = 'female' THEN 'f' ELSE 'm' END NULLS FIRST LIMIT 500"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("ORDER BY \"case_when_a_cust\""));

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );
}

#[tokio::test]
async fn test_case_wrapper_ungrouped() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT CASE WHEN customer_gender = 'female' THEN 'f' ELSE 'm' END, AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("CASE WHEN"));

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );
}

#[tokio::test]
async fn test_case_wrapper_non_strict_match() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let mut config = ConfigObjImpl::default();

    config.disable_strict_agg_type_match = true;

    let query_plan = convert_select_to_query_plan_with_config(
        "SELECT CASE WHEN customer_gender = 'female' THEN 'f' ELSE 'm' END, SUM(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
        Arc::new(config)
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("CASE WHEN"));

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );
}

#[tokio::test]
async fn test_case_wrapper_ungrouped_sorted() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT CASE WHEN customer_gender = 'female' THEN 'f' ELSE 'm' END, AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1 ORDER BY 1 DESC"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("ORDER BY"));
}

#[tokio::test]
async fn test_case_wrapper_ungrouped_sorted_aliased() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT x FROM (SELECT CASE WHEN customer_gender = 'female' THEN 'f' ELSE 'm' END x, AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1 ORDER BY 1 DESC) b"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        // TODO test without depend on column name
        .contains("ORDER BY \"case_when"));
}

#[tokio::test]
async fn test_case_wrapper_with_internal_limit() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT CASE WHEN customer_gender = 'female' THEN 'f' ELSE 'm' END, AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1 LIMIT 1123"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("CASE WHEN"));

    assert!(
        logical_plan
            .find_cube_scan_wrapper()
            .wrapped_sql
            .unwrap()
            .sql
            .contains("1123"),
        "SQL contains 1123: {}",
        logical_plan
            .find_cube_scan_wrapper()
            .wrapped_sql
            .unwrap()
            .sql
    );

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );
}

#[tokio::test]
async fn test_case_wrapper_with_system_fields() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT CASE WHEN customer_gender = 'female' THEN 'f' ELSE 'm' END, __user, __cubeJoinField, AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1, 2, 3 LIMIT 1123"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();

    assert!(
        logical_plan
            .find_cube_scan_wrapper()
            .wrapped_sql
            .unwrap()
            .sql
            .contains(
                "\\\"cube_name\\\":\\\"KibanaSampleDataEcommerce\\\",\\\"alias\\\":\\\"user\\\""
            ),
        r#"SQL contains `\"cube_name\":\"KibanaSampleDataEcommerce\",\"alias\":\"user\"` {}"#,
        logical_plan
            .find_cube_scan_wrapper()
            .wrapped_sql
            .unwrap()
            .sql
    );

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );
}

#[tokio::test]
async fn test_case_wrapper_with_limit() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT * FROM (SELECT CASE WHEN customer_gender = 'female' THEN 'f' ELSE 'm' END, AVG(avgPrice) mp FROM KibanaSampleDataEcommerce a GROUP BY 1) q LIMIT 1123"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("CASE WHEN"));

    assert!(
        logical_plan
            .find_cube_scan_wrapper()
            .wrapped_sql
            .unwrap()
            .sql
            .contains("1123"),
        "SQL contains 1123: {}",
        logical_plan
            .find_cube_scan_wrapper()
            .wrapped_sql
            .unwrap()
            .sql
    );

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );
}

#[tokio::test]
async fn test_case_wrapper_with_null() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT CASE WHEN taxful_total_price IS NULL THEN NULL WHEN taxful_total_price < taxful_total_price * 2 THEN COALESCE(taxful_total_price, 0, 0) END, AVG(avgPrice) FROM KibanaSampleDataEcommerce GROUP BY 1"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("CASE WHEN"));

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );
}

#[tokio::test]
async fn test_case_wrapper_ungrouped_on_dimension() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        "SELECT CASE WHEN SUM(taxful_total_price) > 0 THEN SUM(taxful_total_price) ELSE 0 END FROM KibanaSampleDataEcommerce a"
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );
}

#[tokio::test]
async fn test_case_wrapper_escaping() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan_customized(
        "SELECT CASE WHEN customer_gender = '\\`' THEN COALESCE(customer_gender, 'N/A', 'NN') ELSE 'N/A' END as \"\\`\", AVG(avgPrice) FROM KibanaSampleDataEcommerce a GROUP BY 1".to_string(),
        DatabaseProtocol::PostgreSQL,
        vec![
            ("expressions/binary".to_string(), "{{ left }} \\`{{ op }} {{ right }}".to_string())
        ],
    ).await;

    let physical_plan = query_plan.as_physical_plan().await.unwrap();
    println!(
        "Physical plan: {}",
        displayable(physical_plan.as_ref()).indent()
    );

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        // Expect 6 backslashes as output is JSON and it's escaped one more time
        .contains("\\\\\\\\\\\\`"));
}

/// Test that WrappedSelect(... limit=Some(0) ...) will render it correctly
#[tokio::test]
async fn test_wrapper_limit_zero() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        // language=PostgreSQL
        r#"
            SELECT
                MIN(t.a)
            FROM (
                SELECT
                    MAX(order_date) AS a
                FROM
                    KibanaSampleDataEcommerce
                LIMIT 10
            ) t LIMIT 0
            "#
        .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
    .await;

    let logical_plan = query_plan.as_logical_plan();
    assert!(logical_plan
        .find_cube_scan_wrapper()
        .wrapped_sql
        .unwrap()
        .sql
        .contains("LIMIT 0"));

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

/// Tests that Aggregation(Filter(CubeScan(ungrouped=true))) with expresions in filter
/// can be executed as a single ungrouped=false load query
#[tokio::test]
async fn test_wrapper_filter_flatten() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        // language=PostgreSQL
        r#"
            SELECT
                customer_gender,
                SUM(sumPrice)
            FROM
                KibanaSampleDataEcommerce
            WHERE
                LOWER(customer_gender) = 'male'
            GROUP BY
                1
            "#
        .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
    .await;

    assert_eq!(
        query_plan
            .as_logical_plan()
            .find_cube_scan_wrapper()
            .request
            .unwrap(),
        TransportLoadRequestQuery {
            measures: Some(vec![json!({
                "cube_name": "KibanaSampleDataEcommerce",
                "alias": "sum_kibanasample",
                "cube_params": ["KibanaSampleDataEcommerce"],
                // This SUM(sumPrice) is invalid in grouped query
                "expr": "SUM(${KibanaSampleDataEcommerce.sumPrice})",
                "grouping_set": null,
            })
            .to_string(),]),
            dimensions: Some(vec![json!({
                "cube_name": "KibanaSampleDataEcommerce",
                "alias": "customer_gender",
                "cube_params": ["KibanaSampleDataEcommerce"],
                "expr": "${KibanaSampleDataEcommerce.customer_gender}",
                "grouping_set": null,
            })
            .to_string(),]),
            segments: Some(vec![json!({
                "cube_name": "KibanaSampleDataEcommerce",
                "alias": "lower_kibanasamp",
                "cube_params": ["KibanaSampleDataEcommerce"],
                "expr": "(LOWER(${KibanaSampleDataEcommerce.customer_gender}) = $0$)",
                "grouping_set": null,
            })
            .to_string(),]),
            time_dimensions: None,
            order: Some(vec![]),
            limit: Some(50000),
            offset: None,
            filters: None,
            ungrouped: None,
        }
    );

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}

// There's a lot of breakage due to "aggregate function calls cannot be nested"
// TODO debug
// TODO add test like that

// TODO recheck this, it seems that specific cube (KibanaSampleDataEcommerce vs MultiTypeCube) can influence extraction
#[tokio::test]
async fn test_wrapper_double_agg_filter_flatten() {
    if !Rewriter::sql_push_down_enabled() {
        return;
    }
    init_testing_logger();

    let query_plan = convert_select_to_query_plan(
        // language=PostgreSQL
        r#"-- SELECT
--     "Staples"."dim_str2" AS "Prod Type2",
--     CAST(TRUNC(EXTRACT(MONTH FROM "Staples"."dim_date1")) AS INTEGER) AS "mn:Order Date:ok"
-- --     AVG("Staples"."avgPrice") AS "sum:Gross Profit:ok"
-- --     COUNT(*) as c
-- FROM "MultiTypeCube" "Staples"
-- WHERE
-- --     (CAST(TRUNC(EXTRACT(MONTH FROM "Staples"."dim_date1")) AS INTEGER) <= 8)
--     (Staples.dim_num1 % 100) <= 8
-- GROUP BY
--     1,
--     2

-- SELECT
--     "MultiTypeCube"."dim_str2" AS "Prod Type2",
--     CAST(TRUNC(EXTRACT(MONTH FROM "MultiTypeCube"."dim_date1")) AS INTEGER) AS "mn:Order Date:ok",
--     AVG("MultiTypeCube"."avgPrice") AS "sum:Gross Profit:ok"
-- FROM "MultiTypeCube" "MultiTypeCube"
-- WHERE
--     (CAST(TRUNC(EXTRACT(MONTH FROM "MultiTypeCube"."dim_date1")) AS INTEGER) <= 8)
-- GROUP BY
--     1,
--     2

SELECT
    customer_gender,
    CAST(TRUNC(EXTRACT(MONTH FROM order_date)) AS INTEGER),
    AVG(avgPrice)
FROM
    KibanaSampleDataEcommerce
WHERE
    CAST(TRUNC(EXTRACT(MONTH FROM order_date)) AS INTEGER) <= 8
GROUP BY
    1, 2
"#
            .to_string(),
        DatabaseProtocol::PostgreSQL,
    )
        .await;

    assert_eq!(
        query_plan
            .as_logical_plan()
            .find_cube_scan_wrapper()
            .request
            .unwrap(),
        TransportLoadRequestQuery {
            measures: Some(vec![json!({
                "cube_name": "KibanaSampleDataEcommerce",
                "alias": "sum_kibanasample",
                "cube_params": ["KibanaSampleDataEcommerce"],
                "expr": "SUM(${KibanaSampleDataEcommerce.sumPrice})",
                "grouping_set": null,
            })
            .to_string(),]),
            dimensions: Some(vec![json!({
                "cube_name": "KibanaSampleDataEcommerce",
                "alias": "customer_gender",
                "cube_params": ["KibanaSampleDataEcommerce"],
                "expr": "${KibanaSampleDataEcommerce.customer_gender}",
                "grouping_set": null,
            })
            .to_string(),]),
            segments: Some(vec![json!({
                "cube_name": "KibanaSampleDataEcommerce",
                "alias": "lower_kibanasamp",
                "cube_params": ["KibanaSampleDataEcommerce"],
                "expr": "(LOWER(${KibanaSampleDataEcommerce.customer_gender}) = $0$)",
                "grouping_set": null,
            })
            .to_string(),]),
            time_dimensions: None,
            order: Some(vec![]),
            limit: Some(50000),
            offset: None,
            filters: None,
            ungrouped: None,
        }
    );

    let _physical_plan = query_plan.as_physical_plan().await.unwrap();
}
