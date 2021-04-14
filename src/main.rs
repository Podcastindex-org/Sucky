use std::env;
use std::str;
use std::error::Error;
use std::fmt;
use std::time::SystemTime;
use rusqlite::{ Connection };
use reqwest::header;
use chrono::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use quick_xml::Reader;
use quick_xml::events::Event;

//##: Global definitions
static USERAGENT: &str = "Sucky (PodcastIndex.org)/v0.2";
#[derive(Debug)]
pub struct Item {
    enclosure: String
}
#[derive(Debug)]
struct HydraError(String);

//##: Implement
impl fmt::Display for HydraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fatal error: {}", self.0)
    }
}
impl Error for HydraError {}


//##: -------------------- Main() -----------------------
//##: ---------------------------------------------------
fn main() {
    //Globals
    let args: Vec<String> = env::args().collect();

    //Get the feed url from the command line
    let feed_url = &args[1];
    
    //Download the feed and parse for item enclosures
    match fetch_feed(feed_url) {
        Ok(feed_file_path) => {
            println!("Feed file: [{}]", feed_file_path);

            //Read the feed file
            let mut reader = Reader::from_file(feed_file_path).unwrap();
            //reader.trim_text(true);
            let mut count = 0;
            let mut txt = Vec::new();
            let mut buf = Vec::new();
            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Start(ref e)) => {
                        // if str::from_utf8(e.name()).unwrap() == "title" {
                        //     println!("Episode: {}");
                        // }
                    },
                    Ok(Event::Empty(ref e)) => {
                        //println!("Element: {}", str::from_utf8(e.name()).unwrap());
                        if str::from_utf8(e.name()).unwrap() == "enclosure" {
                            for a in e.attributes() {
                                //println!("  {:?}", a);
                                let attribute = a.unwrap().clone();
                                let ak = attribute.key;
                                let mut av = attribute.value;
                                let attribute_key = str::from_utf8(ak).unwrap();
                                let attribute_value = str::from_utf8(av.to_mut()).unwrap();
                                if attribute_key == "url" {
                                    count = count + 1;
                                    println!("  Enclosure {} url: {}", count, attribute_value);
                                    let mut savedfile: String;
                                    fetch_enclosure(attribute_value, count);
                                }
                            }

                            // e.attributes().map(|a| {
                            //     let attribute = a.unwrap().clone();
                            //     let ak = attribute.key;
                            //     let mut av = attribute.value;
                            //     let attribute_key = str::from_utf8(ak).unwrap();
                            //     let attribute_value = str::from_utf8(av.to_mut()).unwrap();
                            //     if attribute_key == "enclosure" {
                            //         println!("Enclosure url: {}", attribute_value);
                            //         //fetch_enclosure(attribute_value);
                            //     }

                            // }).collect::<Vec<_>>();
                        }
                    },
                    Ok(Event::Text(e)) => txt.push(e.unescape_and_decode(&reader).unwrap()),
                    Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                    Ok(Event::Eof) => break,
                    _ => (),
                }
                buf.clear();
            }

        },
        Err(e) => {
            eprintln!("Error downloading feed. {:?}", e);
        }
    }


}
//##: ---------------------------------------------------


//##: Fetch the content of a url
fn fetch_feed(url: &str) -> Result<&str, Box<dyn Error>> {
    let feed_url: &str = url;

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static(USERAGENT));
    let client = reqwest::blocking::Client::builder().default_headers(headers).build().unwrap();

    //##: Send the request and display the results or the error
    let res = client.get(feed_url).send();
    match res {
        Ok(mut response) => {
            println!("Response Status: [{}]", response.status());
            let mut file = File::create("feed.xml")?;
            response.copy_to(&mut file);
            return Ok("feed.xml");
        },
        Err(e) => {
            eprintln!("Error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error running SQL query: [{}]", e).into())));
        }
    }

}


//##: Fetch an enclosure
fn fetch_enclosure(url: &str, filename_hint: u64) -> Result<bool, Box<dyn Error>> {
    let enclosure_url: &str = url;
    let mut filename: String = "enclosure.".to_owned();
    filename.push_str(filename_hint.to_string().as_str());

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static(USERAGENT));
    let client = reqwest::blocking::Client::builder().default_headers(headers).build().unwrap();

    //##: Send the request and display the results or the error
    let res = client.get(enclosure_url).send();
    match res {
        Ok(mut response) => {
            //Build a filename from the response data
            let fname = response
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or(filename.as_str());

                println!("  Downloading file: {}", fname);

            let mut file = File::create(fname)?;
            response.copy_to(&mut file);

            println!("  Response Status: [{}]", response.status());

            return Ok(true);
        },
        Err(e) => {
            eprintln!("Error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error running SQL query: [{}]", e).into())));
        }
    }

}
