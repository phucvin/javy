pub struct JsModule {
    js_bytecode: Vec<u8>,
}

impl JsModule {
    pub fn new(js_bytecode: Vec<u8>) -> JsModule {
        Self {
            js_bytecode
        }
    }

    pub fn to_wat(&self) -> String {
        let mut tera = tera::Tera::default();
        tera.add_raw_template(
            "js_module.wat",
            std::include_str!("templates/js_module.wat"),
        )
        .unwrap();

        let js_bytes_escaped = base64::encode(&self.js_bytecode);
        // let js_bytes_escaped: String = self.js_bytecode.iter().map(|b| format!("\\{:02X?}", b)).collect();

        let mut context = tera::Context::new();
        context.insert("js_string_length_bytes", &js_bytes_escaped.len());
        context.insert("js_string_bytes", &js_bytes_escaped);
        tera.render("js_module.wat", &context).unwrap()
    }
}