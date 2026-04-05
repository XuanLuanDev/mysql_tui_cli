use mysql::prelude::*;
use mysql::*;

pub trait DatabaseExecutor {
    fn execute(&mut self, query: &str) -> std::result::Result<(Vec<String>, Vec<Vec<String>>), String>;
    fn connect(&mut self, connection_string: &str) -> std::result::Result<(), String>;
    fn get_tables(&self) -> std::result::Result<Vec<String>, String>;
    fn get_databases(&self) -> std::result::Result<Vec<String>, String>;
}

pub struct MysqlExecutor {
    pool: Option<Pool>,
}

impl MysqlExecutor {
    pub fn new() -> Self {
        Self { pool: None }
    }
}

impl DatabaseExecutor for MysqlExecutor {
    fn connect(&mut self, connection_string: &str) -> std::result::Result<(), String> {
        let opts = Opts::from_url(connection_string).map_err(|e| e.to_string())?;
        let pool = Pool::new(opts).map_err(|e| e.to_string())?;
        let mut conn = pool.get_conn().map_err(|e| e.to_string())?;
        let _: Option<i32> = conn.query_first("SELECT 1").map_err(|e| e.to_string())?;
        self.pool = Some(pool);
        Ok(())
    }

    fn execute(&mut self, query: &str) -> std::result::Result<(Vec<String>, Vec<Vec<String>>), String> {
        if query.trim().is_empty() {
            return Ok((vec![], vec![]));
        }
        let pool = self.pool.as_ref().ok_or("Not connected")?;
        let mut conn = pool.get_conn().map_err(|e| e.to_string())?;

        let query_result = conn.query_iter(query).map_err(|e| e.to_string())?;

        let mut columns: Vec<String> = vec![];
        for col in query_result.columns().as_ref() {
            columns.push(col.name_str().into_owned());
        }

        let mut rows_data = vec![];
        
        // Loop through rows
        for row_result in query_result {
            let row = row_result.map_err(|e| e.to_string())?;
            let mut row_out: Vec<String> = vec![];
            
            for i in 0..row.len() {
                let val: Option<String> = row.get(i);
                match val {
                    Some(v) => row_out.push(v),
                    None => {
                        // Sometimes it's NULL, sometimes it's because `get<String>` fails 
                        // due to types like DateTime or raw bytes. We fallback to manual formatting.
                        let raw_val: Option<Value> = row.get(i);
                        match raw_val {
                            Some(Value::Bytes(b)) => row_out.push(String::from_utf8_lossy(&b).to_string()),
                            Some(Value::Int(i)) => row_out.push(i.to_string()),
                            Some(Value::UInt(u)) => row_out.push(u.to_string()),
                            Some(Value::Float(f)) => row_out.push(f.to_string()),
                            Some(Value::Double(d)) => row_out.push(d.to_string()),
                            Some(Value::Date(y, m, d, h, mn, s, _)) => row_out.push(format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, h, mn, s)),
                            Some(Value::Time(is_neg, d, h, m, s, _)) => {
                                let sign = if is_neg { "-" } else { "" };
                                row_out.push(format!("{}{} days {:02}:{:02}:{:02}", sign, d, h, m, s))
                            },
                            Some(Value::NULL) => row_out.push("NULL".to_string()),
                            None => row_out.push("NULL".to_string()),
                        }
                    }
                }
            }
            rows_data.push(row_out);
        }

        Ok((columns, rows_data))
    }

    fn get_tables(&self) -> std::result::Result<Vec<String>, String> {
        let pool = self.pool.as_ref().ok_or("Not connected")?;
        let mut conn = pool.get_conn().map_err(|e| e.to_string())?;

        let tables: Vec<String> = conn.query("SHOW TABLES").map_err(|e| e.to_string())?;
        Ok(tables)
    }

    fn get_databases(&self) -> std::result::Result<Vec<String>, String> {
        let pool = self.pool.as_ref().ok_or("Not connected")?;
        let mut conn = pool.get_conn().map_err(|e| e.to_string())?;

        let dbs: Vec<String> = conn.query("SHOW DATABASES").map_err(|e| e.to_string())?;
        Ok(dbs)
    }
}
