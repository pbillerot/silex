///
/// CRUD sur les données
///
use sqlx::{Pool, Postgres, Sqlite};

use crate::lexicer::lex_table::{Element, Table};
use std::collections::HashMap;
///
/// - Mise à jour d'un article
///
pub async fn crud_update(
    pooldb: &Pool<Postgres>,
    poolite: &Pool<Sqlite>,
    table: &Table, // le lexique de l'application
    velements: &Vec<Element>,
    id: &str,
    form_posted: &Vec<(String, String)>,
) -> Result<String, String> {
    // Transformation de form_posted Vec(key, value) en Hashtable
    // sachant key ne sera unique pour les "select multiple" === tag
    let mut hvalue: HashMap<String, String> = HashMap::new();
    let mut key = String::new();
    let mut val = String::new();
    for (k, v) in form_posted {
        if key.is_empty() {
            key = k.clone();
        }
        if &key == k {
            if !val.is_empty() {
                val.push_str(", ");
            }
            val.push_str(v);
        } else {
            hvalue.insert(key.clone(), val.clone());
            key = k.clone();
            val.clear();
            val.push_str(v);
        }
    }
    hvalue.insert(key, val.clone());

    // valorisation des éléments du formulaire avec les champs du formulaire
    // construction de l'order sql
    // construction de l'ordre sql de mise à jour
    let mut sql = "UPDATE ".to_string();
    sql.push_str(&table.tableid);
    sql.push_str(" SET ");
    // on prend les colonnes définies dans la view.velements
    let mut bstart = true;
    for vel in velements {
        let mut element = vel.clone();
        match vel.type_element.as_str() {
            "checkbox" => {
                // 'on' si coché et rien si non coché
                element.value = match hvalue.get(&vel.elid) {
                    Some(_) => "1".to_string(),
                    None => "0".to_string(),
                };
            }
            _ => {
                element.value = match hvalue.get(&vel.elid) {
                    Some(v) => v.to_string(),
                    None => String::new(),
                };
            }
        }
        element.compute_prop(pooldb, poolite, &hvalue, false).await?;
        element.key_value = id.to_string();
        // construction du sql
        if element.elid == table.setting.key {
            continue;
        }
        if element.type_element == "counter" {
            continue;
        }
        if element.hide {
            continue;
        }
        if element.elid.starts_with("_") {
            continue;
        }
        if bstart {
            bstart = false;
        } else {
            sql.push_str(", ");
        }
        sql.push_str(format!("{} = '{}'", &element.elid, element.value.replace("'", "''")).as_str());
    }
    sql.push_str(format!(" WHERE ( {} = '{}' )", &table.setting.key, &id).as_str());

    let result = match sqlx::query(&sql).execute(pooldb).await {
        Ok(r) => r,
        Err(e) => {
            let msg = format!("{sql:?} : {e:?}");
            log::error!("{msg}");
            return Err(msg)
        }
    };

    Ok(format!("{sql:?} {result:?}"))

}

