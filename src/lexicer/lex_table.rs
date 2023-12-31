use crate::cruder::sqler::{kerdata, kerlite};
use crate::lexicer::lex_utils;
use serde::{Deserialize, Serialize};
use serde_yaml::{self};
use sqlx::{Pool, Postgres, Sqlite};
use std::collections::HashMap;

use super::macvalue;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Table {
    #[serde(default = "String::new")]
    pub tableid: String,
    #[serde(default = "Setting::new")]
    pub setting: Setting,
    #[serde(default = "HashMap::new")]
    pub elements: HashMap<String, Element>,
    #[serde(default = "HashMap::new")]
    pub views: HashMap<String, View>,
    #[serde(default = "HashMap::new")]
    pub forms: HashMap<String, Form>,
    #[serde(default = "HashMap::new")]
    pub variables: HashMap<String, String>,
}

impl Table {
    pub fn load(appid: &str, tableid: &str) -> Result<Table, String> {
        let lexic_path = std::env::var("LEXIC_PATH")
            .map_err(|e| format!("Unable to read LEXIC_PATH env var {:?}", e))?;
        let path = format!("{}/{}/config/{}.yaml", &lexic_path, appid, tableid);
        log::info!("Load de {}", path);
        let f = std::fs::File::open(&path).map_err(|e| format!("Could not open file {:?}", e))?;
        let mut table: Table =
            serde_yaml::from_reader(f).map_err(|e| format!("Could not read values {:?}", e))?;
        table.tableid = tableid.to_string().clone();
        // alimentation de velements avec view.elements fusionnés avec table.elements
        for (viewid, view) in table.views.iter_mut() {
            view.tableid = tableid.to_string().clone();
            view.viewid = viewid.clone();
            for (key, element) in &view.elements {
                let mut el = element.clone();
                match table.elements.get(key) {
                    Some(t) => {
                        el.elid = key.clone();
                        el.merge(t);
                        el.init_prop();
                        view.velements.push(el);
                    }
                    None => {
                        // un view.elements peut ne pas exister dans table.elements
                        el.elid = key.clone();
                        el.init_prop();
                        view.velements.push(el);
                    }
                };
            }
            view.velements.sort_by(|a, b| a.order.cmp(&b.order));
        }
        // alimentation de velements avec form.elements fusionnés avec table.elements
        for (formid, form) in table.forms.iter_mut() {
            form.formid = formid.clone(); // TODO: que devient l'ancienne valeur ?
            for (key, element) in &form.elements {
                let mut el = element.clone();
                match table.elements.get(key) {
                    Some(t) => {
                        el.elid = key.clone();
                        el.merge(t);
                        el.init_prop();
                        form.velements.push(el);
                    }
                    None => {
                        // un form.elements peut ne pas exister dans table.elements
                        el.elid = key.clone();
                        el.init_prop();
                        form.velements.push(el);
                    }
                };
            }
            form.velements.sort_by(|a, b| a.order.cmp(&b.order));
        }

        Ok(table)
    }
}

// Element as
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Element {
    #[serde(default = "String::new")]
    pub elid: String,
    #[serde(default = "Vec::new")]
    pub actions: Vec<Action>, // bouton d'actions
    #[serde(default = "HashMap::new")]
    pub args: HashMap<String, String>, // Args pour passer des arguments à une vue
    #[serde(default = "String::new")]
    pub ajax_sql: String, // query sql pour ramenener des données dans le formulaire
    #[serde(default = "String::new")]
    pub class: String, // class semantic
    #[serde(default = "String::new")]
    pub class_sqlite: String, // SQL pour alimenter Class error warning info green blue
    #[serde(default = "String::new")]
    pub col_align: String, //
    #[serde(default = "lex_utils::default_bool")]
    pub col_no_wrap: bool, // nowrap de la colonne
    #[serde(default = "HashMap::new")]
    pub dataset: HashMap<String, String>, // Dataset pour un Chartjs ou pour passer des arguments à une vue ou à une "ajax-sql"
    #[serde(default = "String::new")]
    pub default: String, // valeur par défaut
    #[serde(default = "String::new")]
    pub default_sqlite: String, // Ordre SQL qui retournera la colonne pour alimenter Default
    #[serde(default = "String::new")]
    pub error: String, // valeur par défaut
    #[serde(default = "String::new")]
    pub filter: String, // pattern du display
    #[serde(default = "String::new")]
    pub format: String, // pattern du display
    #[serde(default = "String::new")]
    pub format_sqlite: String, // select strftime('%H:%M:%S', {Milliseconds}/1000, 'unixepoch')
    #[serde(default = "String::new")]
    pub group: String, // Groupe autorisé à accéder à cette rubrique - Si owner c'est l'enregistreement qui sera protégé
    #[serde(default = "String::new")]
    pub help: String, // TODO aide sur la rubrique
    #[serde(default = "lex_utils::default_bool")]
    pub hide: bool, // TODO cachée si condition
    #[serde(default = "String::new")]
    pub hide_sqlite: String, // TODO cachée si condition
    #[serde(default = "lex_utils::default_bool")]
    pub hide_on_mobile: bool, // La colonne dans une vue sera cachée sur Mobile
    #[serde(default = "String::new")]
    pub icon_name: String, // icone d'une card par exemple
    #[serde(default = "Vec::new")]
    pub items: Vec<HashMap<String, String>>, // slice d'item
    #[serde(default = "String::new")]
    pub items_sql: String, // Ordre SQL qui retournera la colonne pour alimenter Items
    #[serde(default = "Jointure::new")]
    pub jointure: Jointure, // élément issu d'une jointure SQL (lecture seule)
    #[serde(default = "String::new")]
    pub label_long: String, // Label dans un formulaire
    #[serde(default = "String::new")]
    pub label_short: String, // Label dans une vue
    #[serde(default = "lex_utils::default_i32")]
    pub max: i32, // TODO valeur max
    #[serde(default = "lex_utils::default_i32")]
    pub max_length: i32, // TODO longeur max
    #[serde(default = "lex_utils::default_i32")]
    pub min: i32, // TODO valeur min
    #[serde(default = "lex_utils::default_i32")]
    pub min_length: i32, // TODO longueur min
    #[serde(default = "lex_utils::default_i32")]
    pub order: i32, // Ordre de l'élément dans une vue ou formulaire
    #[serde(default = "Params::new")]
    pub params: Params, // paramètres optionnels
    #[serde(default = "String::new")]
    pub pattern: String, // Pattern de l'input HTML
    #[serde(default = "String::new")]
    pub place_holder: String, // Label dans le champ en saisie si vide
    #[serde(default = "lex_utils::default_bool")]
    pub protected: bool, // Est en mise à jour mais protégé en saisie
    #[serde(default = "lex_utils::default_bool")]
    pub read_only: bool, // Lecteur seule
    #[serde(default = "lex_utils::default_bool")]
    pub required: bool, // obligatoire
    #[serde(default = "String::new")]
    pub sort_direction: String, // "", ascending, ou descending pour demander un tri à la requête sql
    #[serde(default = "String::new")]
    pub sql_out: String, // Valeur à enregistrer dans la base de données (zone calculée par le beedule)
    #[serde(default = "String::new")]
    pub style: String, // style de la cellule
    #[serde(default = "String::new")]
    pub style_sqlite: String, // style de la cellule
    #[serde(default = "lex_utils::default_f64")]
    pub sum: f64, // somme des valeurs de la colonne de la table
    #[serde(default = "String::new")]
    pub type_element: String, // Type : amount button card chart checkbox counter date email float image list number password pdf percent tag tel text textarea time radio url
    #[serde(default = "String::new")]
    pub width: String, // largeur s m l xl xxl max 150px 360px 450px 600px 750px 100% dans view et edit	WithSum       bool              // dans une table calcule la somme des valeurs
    #[serde(default = "lex_utils::default_bool")]
    pub with_script: bool, // javascript de présentation
    #[serde(default = "lex_utils::default_bool")]
    pub with_sum: bool, // dans une table calcule la somme des valeurs
    // calcul
    #[serde(default = "String::new")]
    pub value: String, // valeur récupérée dans la table des données
    #[serde(default = "String::new")]
    pub key_value: String, // valeur de la clé de l'enregistrement correspondant
}

impl Element {
    /// Initialisation des valeurs par défaut des propriétés de l'élément sans données contextuelles
    pub fn init_prop(&mut self) {
        if self.place_holder.is_empty() {
            if !self.label_long.is_empty() {
                self.place_holder = self.label_long.clone();
            } else if !self.label_short.is_empty() {
                self.place_holder = self.label_short.clone();
            } else {
                self.place_holder = self.elid.clone();
            }
        }
    }

    /// Calcul de la valeur de l'élément et de ses propriétés à partir des données lues dans la table
    pub async fn compute_value(
        &mut self,
        poolite: &Pool<Sqlite>,
        hvalue: &HashMap<String, String>,
    ) -> Result<&mut Self, String> {
        // get value dans la table
        if !self.elid.starts_with("_") {
            match hvalue.get(&self.elid) {
                Some(v) => self.value = v.clone(),
                None => return Err(format!("Colonne {} non trouvée", self.elid)),
            }
            // self.value = hvalue.get(&self.elid).unwrap().clone();
        }
        // valeur par défaut
        if self.value.is_empty() && !self.default_sqlite.is_empty() {
            let sql = macvalue(&self.default_sqlite, hvalue);
            self.value = kerlite(poolite, &sql).await?;
        }
        if self.value.is_empty() && !self.default.is_empty() {
            self.value = macvalue(&self.default, hvalue);
        }
        Ok(self)
    }

    pub async fn compute_prop(
        &mut self,
        pooldb: &Pool<Postgres>,
        poolite: &Pool<Sqlite>,
        hvalue: &HashMap<String, String>,
        compute_list: bool,
    ) -> Result<&mut Self, String> {
        // valeur par défaut
        if !self.default.is_empty() {
            self.default = macvalue(&self.default, hvalue);
        }
        if !self.default_sqlite.is_empty() {
            let sql = macvalue(&self.default_sqlite, hvalue);
            self.default = kerlite(poolite, &sql).await?;
        }
        if self.value.is_empty() {
            self.value = self.default.clone();
        }

        // Macrolex des autres propriétés
        if !self.label_long.is_empty() {
            self.label_long = macvalue(&self.label_long, hvalue);
        }
        if !self.label_short.is_empty() {
            self.label_short = macvalue(&self.label_short, hvalue);
        }
        if !self.place_holder.is_empty() {
            self.place_holder = macvalue(&self.place_holder, hvalue);
        }
        if self.place_holder.is_empty() {
            if self.label_long.is_empty() {
                self.place_holder = self.label_long.clone();
            } else {
                self.place_holder = self.label_short.clone();
            }
        }
        if !self.help.is_empty() {
            self.help = macvalue(&self.help, hvalue);
        }
        if !self.params.url.is_empty() {
            self.params.url = macvalue(&self.params.url, hvalue);
        }

        // macrolex suivi de kerlite
        if !self.class_sqlite.is_empty() {
            let sql = macvalue(&self.class_sqlite, hvalue);
            self.class = kerlite(poolite, &sql).await?;
        }
        if !self.format_sqlite.is_empty() {
            let sql = macvalue(&self.format_sqlite, hvalue);
            self.format = kerlite(poolite, &sql).await?;
        }
        if !self.hide_sqlite.is_empty() {
            let sql = macvalue(&self.hide_sqlite, hvalue);
            self.hide = !kerlite(poolite, &sql).await?.is_empty();
        }
        if !self.style_sqlite.is_empty() {
            let sql = macvalue(&self.style_sqlite, hvalue);
            self.style = kerlite(poolite, &sql).await?;
        }
        // items récupérés dans les données de l'application
        if compute_list {
            if !self.items_sql.is_empty() {
                let sql = macvalue(&self.items_sql, hvalue);
                self.items = kerdata(pooldb, &sql).await?;
            }
        }
        if !self.params.title.is_empty() {
            self.params.title = macvalue(&self.params.title, hvalue);
        }
        if !self.params.src.is_empty() {
            self.params.src = macvalue(&self.params.src, hvalue);
        }
        if !self.params.url.is_empty() {
            self.params.url = macvalue(&self.params.url, hvalue);
        }

        if self.col_no_wrap {
            if self.class.is_empty() {
                self.class = "crud-cell-nowrap".into();
            } else {
                self.class.push_str(" crud-cell-nowrap");
            }
        }

        // Calcul des propriétés en fonction du contexte
        // format_spec := [[fill]align][sign]['#']['0'][width]['.' precision]type
        match self.type_element.as_str() {
            "amount" => {
                if self.format.is_empty() {
                    if !self.value.is_empty() {
                        self.format = format!("{:.2} €", self.value.parse::<f64>().unwrap());
                    }
                    self.col_align = "right".into();
                    if self.class.is_empty() {
                        self.class = "crud-cell-nowrap".into();
                    }
                }
            }
            "counter" => {
                self.read_only = true;
                self.required = false;
                if self.col_align.is_empty() {
                    self.col_align = "center".into();
                }
            }
            _ => {}
        }

        Ok(self)
    }

    /// fusion des propriétés éléments de la vue ou formulaire avec les élement déclarés au niveau de la table (tel)
    fn merge(&mut self, tel: &Element) {
        // let mut fusel = fullElement;
        if self.elid.is_empty() {
            self.elid = tel.elid.clone();
        }
        if self.actions.is_empty() {
            self.actions = tel.actions.clone();
        }
        if self.args.is_empty() {
            self.args = tel.args.clone();
        }
        if self.ajax_sql.is_empty() {
            self.ajax_sql = tel.ajax_sql.clone();
        }
        if self.class_sqlite.is_empty() {
            self.class_sqlite = tel.class_sqlite.clone();
        }
        if self.col_align.is_empty() {
            self.col_align = tel.col_align.clone();
        }
        if tel.col_no_wrap == true {
            self.col_no_wrap = true;
        }
        if self.dataset.is_empty() {
            self.dataset = tel.dataset.clone();
        }
        if self.default.is_empty() {
            self.default = tel.default.clone();
        }
        if self.default_sqlite.is_empty() {
            self.default_sqlite = tel.default_sqlite.clone();
        }
        if self.format_sqlite.is_empty() {
            self.format_sqlite = tel.format_sqlite.clone();
        }
        if self.group.is_empty() {
            self.group = tel.group.clone();
        }
        if self.help.is_empty() {
            self.help = tel.help.clone();
        }
        if tel.hide == true {
            self.hide = true;
        }
        if self.hide_sqlite.is_empty() {
            self.hide_sqlite = tel.hide_sqlite.clone();
        }
        if tel.hide_on_mobile == true {
            self.hide_on_mobile = true;
        }
        if self.icon_name.is_empty() {
            self.icon_name = tel.icon_name.clone();
        }
        if self.items.is_empty() {
            self.items = tel.items.clone();
        }
        if self.items_sql.is_empty() {
            self.items_sql = tel.items_sql.clone();
        }
        if self.jointure.column.is_empty() {
            self.jointure = tel.jointure.clone();
        }
        if self.label_long.is_empty() {
            self.label_long = tel.label_long.clone();
        }
        if self.label_short.is_empty() {
            self.label_short = tel.label_short.clone();
        }
        if self.max == 0 {
            self.max = tel.max;
        }
        if self.max_length == 0 {
            self.max_length = tel.max_length;
        }
        if self.min == 0 {
            self.min = tel.min;
        }
        if self.min_length == 0 {
            self.min_length = tel.min_length;
        }
        if self.order == 0 {
            self.order = tel.order;
        }

        self.params.merge(&tel.params);

        if self.pattern.is_empty() {
            self.pattern = tel.pattern.clone();
        }
        if self.place_holder.is_empty() {
            self.place_holder = tel.place_holder.clone();
        }
        if tel.protected == true {
            self.protected = true;
        }
        if tel.read_only == true {
            self.read_only = true;
        }
        if tel.required == true {
            self.required = true;
        }
        if self.sort_direction.is_empty() {
            self.sort_direction = tel.sort_direction.clone();
        }
        if self.sql_out.is_empty() {
            self.sql_out = tel.sql_out.clone();
        }
        if self.style_sqlite.is_empty() {
            self.style_sqlite = tel.style_sqlite.clone();
        }
        if self.type_element.is_empty() {
            self.type_element = tel.type_element.clone();
        }
        if tel.with_script == true {
            self.with_script = true;
        }
        if tel.with_sum == true {
            self.with_sum = true;
        }
    }
}

// View Vue d'une table
#[derive(Debug, Serialize, Deserialize)]
pub struct View {
    #[serde(default = "String::new")]
    pub viewid: String,
    #[serde(default = "Action::new")]
    pub action_press: Action, // Action sur appui long sur l'article
    #[serde(default = "Vec::new")]
    pub actions: Vec<Action>, // Action sur la vue (ordres sql)
    #[serde(default = "CardList::new")]
    pub card: CardList, // Masque html d'une ligne dans la vue
    #[serde(default = "String::new")]
    pub class: String, // couleur theme de la ligne
    #[serde(default = "String::new")]
    pub class_sqlite: String, // couleur theme de la ligne
    #[serde(default = "lex_utils::default_bool")]
    pub deletable: bool, // Suppression fiche autorisée
    #[serde(default = "HashMap::new")]
    pub elements: HashMap<String, Element>, // Eléments à récupérer de la base de données
    #[serde(default = "Vec::new")]
    pub filters: Vec<String>, // liste de nom d'élément sur la vue
    #[serde(default = "String::new")]
    pub footer_sql: String, // requête sur la table courante
    #[serde(default = "String::new")]
    pub form_add: String, // Formulaire d'ajout
    #[serde(default = "String::new")]
    pub form_edit: String, // Formulaire d'édition
    #[serde(default = "String::new")]
    pub form_view: String, // Masque de visualisation d'un enregistrement
    #[serde(default = "String::new")]
    pub group: String, // groupe qui peut accéder à la vue
    #[serde(default = "lex_utils::default_bool")]
    pub hide: bool, // Vue cachée dans le menu
    #[serde(default = "lex_utils::default_bool")]
    pub hide_on_mobile: bool, // Vue cachée dur mobile
    #[serde(default = "String::new")]
    pub icon_name: String, // nom de l'icone
    #[serde(default = "lex_utils::default_i32")]
    pub limit: i32, // pour limiter le nbre de ligne dans la vue
    #[serde(default = "String::new")]
    pub order_by: String, // Tri des données SQL
    #[serde(default = "Vec::new")]
    pub post_sql: Vec<String>, // Ordre exécutée après la suppression si OK
    #[serde(default = "String::new")]
    pub search: String, // calculé sql
    #[serde(default = "String::new")]
    pub style: String, // style de la ligne
    #[serde(default = "String::new")]
    pub style_sqlite: String, // style de la ligne
    #[serde(default = "String::new")]
    pub title: String, // Titre de la vue
    #[serde(default = "String::new")]
    pub type_view: String, // type de vue : card(default),image,table
    #[serde(default = "String::new")]
    pub where_sql: String, // Condition SQL
    #[serde(default = "String::new")]
    pub width: String, // largeur s m l xl xxl max
    #[serde(default = "lex_utils::default_bool")]
    pub with_line_number: bool, // list.table n° de ligne en 1ère colonne
    #[serde(default = "lex_utils::default_bool")]
    pub with_sum: bool,
    // calcul
    #[serde(default = "String::new")]
    pub tableid: String, // parent de la vue
    #[serde(default = "Vec::new")]
    pub velements: Vec<Element>,
}

impl Clone for View {
    fn clone(&self) -> View {
        View {
            viewid: self.viewid.clone(),
            action_press: self.action_press.clone(),
            actions: self.actions.clone(),
            card: self.card.clone(),
            class: self.class.clone(),
            class_sqlite: self.class_sqlite.clone(),
            deletable: self.deletable.clone(),
            elements: self.elements.clone(),
            filters: self.filters.clone(),
            footer_sql: self.footer_sql.clone(),
            form_add: self.form_add.clone(),
            form_edit: self.form_edit.clone(),
            form_view: self.form_view.clone(),
            group: self.group.clone(),
            hide: self.hide.clone(),
            hide_on_mobile: self.hide_on_mobile.clone(),
            icon_name: self.icon_name.clone(),
            limit: self.limit.clone(),
            order_by: self.order_by.clone(),
            post_sql: self.post_sql.clone(),
            search: self.search.clone(),
            style: self.style.clone(),
            style_sqlite: self.style_sqlite.clone(),
            title: self.title.clone(),
            type_view: self.type_view.clone(),
            where_sql: self.where_sql.clone(),
            width: self.width.clone(),
            with_line_number: self.with_line_number.clone(),
            with_sum: self.with_sum.clone(),
            tableid: self.tableid.clone(),
            velements: self.velements.clone(),
        }
    }
}
// Form formulaire
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Form {
    #[serde(default = "String::new")]
    pub formid: String,
    #[serde(default = "Vec::new")]
    pub actions: Vec<Action>, // Action appel d'un formulaire ou exécution d'une requête SQL
    #[serde(default = "String::new")]
    pub title: String, // Titre du formulaire
    #[serde(default = "String::new")]
    pub group: String, // groupe qui peut accéder au formulaire
    #[serde(default = "lex_utils::default_bool")]
    pub hide_submit: bool, // pour caher le bouton valider
    #[serde(default = "String::new")]
    pub icon_name: String, // nom de l'icone
    #[serde(default = "HashMap::new")]
    pub elements: HashMap<String, Element>, // Eléments à récupérer de la base de données
    #[serde(default = "Vec::new")]
    pub check_sqlite: Vec<String>, // retourne le libellé des erreurs lors du contrôle des rubriques
    #[serde(default = "Vec::new")]
    pub post_sql: Vec<String>, // Ordre exécutée après la validation si contrôle OK
    // calcul
    #[serde(default = "Vec::new")]
    pub velements: Vec<Element>,
}

// SETTING
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Setting {
    #[serde(default = "String::new")]
    pub alias_db: String,
    #[serde(default = "String::new")]
    pub key: String,
    #[serde(default = "String::new")]
    pub icon_name: String,
}
impl Setting {
    pub fn new() -> Setting {
        Setting {
            alias_db: String::new(),
            key: String::new(),
            icon_name: String::new(),
        }
    }
}

// ACTIONS
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Action {
    #[serde(default = "String::new")]
    pub group: String, // Groupe autorisée à lancer l'action
    #[serde(default = "String::new")]
    pub label: String, // label de l'action
    #[serde(default = "String::new")]
    pub url: String, // URL d'appel du formulaire
    #[serde(default = "Vec::new")]
    pub sql: Vec<String>, // les ordres SQL seront exécutées avant l'appel du formulaire
    #[serde(default = "lex_utils::default_bool")]
    pub with_confirm: bool, // demande de  confirmation
    #[serde(default = "lex_utils::default_bool")]
    pub hide: bool, // Action non visible
    #[serde(default = "String::new")]
    pub hide_sqlite: String, // requête pour cachée l'action
    #[serde(default = "Vec::new")]
    pub shell: Vec<String>, // commande dans le shell
}
impl Action {
    pub fn new() -> Action {
        Action {
            group: String::new(),
            label: String::new(),
            url: String::new(),
            sql: Vec::new(),
            with_confirm: false,
            hide: false,
            hide_sqlite: String::new(),
            shell: Vec::new(),
        }
    }
}

// Params paramètres d'un élément
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Params {
    #[serde(default = "String::new")]
    pub formid: String, // card: form à ouvrir
    #[serde(default = "Vec::new")]
    pub header: Vec<String>, // card pour image
    #[serde(default = "Vec::new")]
    pub description: Vec<String>, // card pour image
    #[serde(default = "Vec::new")]
    pub meta: Vec<String>, // card pour image
    #[serde(default = "Vec::new")]
    pub extra: Vec<String>, // card pour image
    #[serde(default = "String::new")]
    pub url: String,
    #[serde(default = "String::new")]
    pub src: String, // card: src de l'image
    #[serde(default = "Vec::new")]
    pub sql: Vec<String>,
    #[serde(default = "String::new")]
    pub tableid: String, // card:
    #[serde(default = "String::new")]
    pub target: String, // target si URL
    #[serde(default = "String::new")]
    pub title: String, // title sur une image
    #[serde(default = "String::new")]
    pub viewid: String, // card:
    #[serde(default = "String::new")]
    pub where_sql: String, // card: + params.table + params.view
    #[serde(default = "lex_utils::default_bool")]
    pub with_confirm: bool,
    #[serde(default = "lex_utils::default_bool")]
    pub without_frame: bool, // card sans cadre
}
impl Params {
    pub fn new() -> Params {
        Params {
            formid: String::new(),
            header: Vec::new(),
            description: Vec::new(),
            meta: Vec::new(),
            extra: Vec::new(),
            url: String::new(),
            src: String::new(),
            sql: Vec::new(),
            tableid: String::new(),
            target: String::new(),
            title: String::new(),
            viewid: String::new(),
            where_sql: String::new(),
            with_confirm: false,
            without_frame: false,
        }
    }
    pub fn merge(&mut self, telp: &Params) {
        if self.formid.is_empty() {
            self.formid = telp.formid.clone();
        }
        if self.header.is_empty() {
            self.header = telp.header.clone();
        }
        if self.description.is_empty() {
            self.description = telp.description.clone();
        }
        if self.meta.is_empty() {
            self.meta = telp.meta.clone();
        }
        if self.extra.is_empty() {
            self.extra = telp.extra.clone();
        }
        if self.url.is_empty() {
            self.url = telp.url.clone();
        }
        if self.src.is_empty() {
            self.src = telp.src.clone();
        }
        if self.tableid.is_empty() {
            self.tableid = telp.tableid.clone();
        }
        if self.target.is_empty() {
            self.target = telp.target.clone();
        }
        if self.title.is_empty() {
            self.title = telp.title.clone();
        }
        if self.viewid.is_empty() {
            self.viewid = telp.viewid.clone();
        }
        if self.where_sql.is_empty() {
            self.where_sql = telp.where_sql.clone();
        }
        if telp.with_confirm == true {
            self.with_confirm = true;
        }
        if self.without_frame == true {
            self.without_frame = true;
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct CardList {
    #[serde(default = "Vec::new")]
    pub header: Vec<String>,
    #[serde(default = "Vec::new")]
    pub header_right: Vec<String>,
    #[serde(default = "Vec::new")]
    pub header_action: Vec<Action>, // bouton menu d'action
    #[serde(default = "Vec::new")]
    pub meta: Vec<String>,
    #[serde(default = "Vec::new")]
    pub meta_left: Vec<String>,
    #[serde(default = "Vec::new")]
    pub meta_right: Vec<String>,
    #[serde(default = "Vec::new")]
    pub description: Vec<String>,
    #[serde(default = "Vec::new")]
    pub extra: Vec<String>,
    #[serde(default = "Vec::new")]
    pub extra_left: Vec<String>,
    #[serde(default = "Vec::new")]
    pub extra_right: Vec<String>,
    #[serde(default = "Vec::new")]
    pub footer: Vec<String>,
    #[serde(default = "Vec::new")]
    pub footer_left: Vec<String>,
    #[serde(default = "Vec::new")]
    pub footer_right: Vec<String>,
    #[serde(default = "Vec::new")]
    pub footer_action: Vec<Action>, // bouton menu d'action
}
impl CardList {
    pub fn new() -> CardList {
        CardList {
            header: Vec::new(),
            header_right: Vec::new(),
            header_action: Vec::new(),
            meta: Vec::new(),
            meta_left: Vec::new(),
            meta_right: Vec::new(),
            description: Vec::new(),
            extra: Vec::new(),
            extra_left: Vec::new(),
            extra_right: Vec::new(),
            footer: Vec::new(),
            footer_left: Vec::new(),
            footer_right: Vec::new(),
            footer_action: Vec::new(),
        }
    }
}

// Jointure entre tables
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Jointure {
    #[serde(default = "String::new")]
    pub join: String, // la commande du genre : left outer join on field = field
    #[serde(default = "String::new")]
    pub column: String, // colonne retournée par la jointure
}
impl Jointure {
    pub fn new() -> Jointure {
        Jointure {
            join: String::new(),
            column: String::new(),
        }
    }
}
