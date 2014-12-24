use helpers::{HelperDef};
use template::{Helper};
use registry::{Registry};
use context::{Context, JsonTruthy};
use render::{Renderable, RenderContext, RenderError, render_error, EMPTY};

#[deriving(Copy)]
pub struct WithHelper;

impl HelperDef for WithHelper {
    fn resolve(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let param = h.params().get(0);

        if param.is_none() {
            return Err(render_error("Param not found for helper \"with\""));
        }

        let path = rc.get_path().clone();
        rc.promote_local_vars();

        let value = c.navigate(rc.get_path(), param.unwrap());

        let not_empty = value.is_truthy();
        let template = if not_empty {
            h.template()
        } else {
            h.inverse()
        };

        if not_empty {
            let new_path = format!("{}/{}", path, param.unwrap());
            rc.set_path(new_path);
        }

        let rendered = match template {
            Some(t) => t.render(c, r, rc),
            None => Ok(EMPTY.to_string())
        };

        rc.set_path(path);
        rc.demote_local_vars();
        rendered
    }
}

pub static WITH_HELPER: WithHelper = WithHelper;

#[cfg(test)]
mod test {
    use template::{Template};
    use registry::{Registry};
    use helpers::{WITH_HELPER};

    use std::collections::BTreeMap;
    use serialize::json::{Json, ToJson};

    struct Address {
        city: String,
        country: String
    }

    impl ToJson for Address {
        fn to_json(&self) -> Json {
            let mut m = BTreeMap::new();
            m.insert("city".to_string(), self.city.to_json());
            m.insert("country".to_string(), self.country.to_json());
            Json::Object(m)
        }
    }

    struct Person {
        name: String,
        age: i16,
        addr: Address,
        titles: Vec<String>
    }

    impl ToJson for Person {
        fn to_json(&self) -> Json {
            let mut m = BTreeMap::new();
            m.insert("name".to_string(), self.name.to_json());
            m.insert("age".to_string(), self.age.to_json());
            m.insert("addr".to_string(), self.addr.to_json());
            m.insert("titles".to_string(), self.titles.to_json());
            Json::Object(m)
        }
    }

    #[test]
    fn test_with() {
        let addr = Address {
            city: "Beijing".to_string(),
            country: "China".to_string()
        };

        let person = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr: addr,
            titles: vec!["programmer".to_string(),
                         "cartographier".to_string()]
        };

        let t0 = Template::compile("{{#with addr}}{{city}}{{/with}}".to_string()).unwrap();
        let t1 = Template::compile("{{#with notfound}}hello{{else}}world{{/with}}".to_string()).unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", &t0);
        handlebars.register_template("t1", &t1);
        handlebars.register_helper("with", box WITH_HELPER);

        let r0 = handlebars.render("t0", &person);
        assert_eq!(r0.unwrap(), "Beijing".to_string());

        let r1 = handlebars.render("t1", &person);
        assert_eq!(r1.unwrap(), "world".to_string());
    }
}
