use log::{debug, warn, error, log_enabled, info, Level};
use env_logger::Env;
use log4rs;
/* 
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};
*/
fn test_log_levels(){
   // env_logger::init();
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    info!("test case starts : test_log_levels");
    debug!("this is a debug {}", "message");
    error!("this is printed by default");
    if log_enabled!(Level::Info){
        let x = 3 * 4;
        info!("the answer was : {}", x);
    }
    info!("test case ends : test_log_levels");
}

fn test_log_write_file() {

    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();

    /* 

    let stdout = ConsoleAppender::builder().build();

    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("log/requests.log")
        .unwrap();

     
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("requests", Box::new(requests)))
        .logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
        .logger(Logger::builder()
            .appender("requests")
            .additive(false)
            .build("app::requests", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();

        let _handle = log4rs::init_config(config)?;
         */

    // use handle to change logger configuration at runtime
    info!("test cases starts: test_log_write_file"); 
    error!("error goes to stderr and file");
    warn!("warn goes to stderr and file");
    debug!("debug goes to file only");
    info!("test cases ends: test_log_write_file");


}

#[cfg(test)]
mod tests {
    use crate::{test_log_levels, test_log_write_file};


    #[test]
    fn test_run_log_info() {
       test_log_levels();
        assert_eq!(1+2, 3);
    }

    #[test]
    fn test_run_log_write_file() {
        test_log_write_file();
         assert_eq!(1+2, 3);
     }

}