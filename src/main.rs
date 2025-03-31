use curl::easy::Easy;

use std::env;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io::{Write, stdin, stdout};

const AGENT: &str = "rwt/0.0.2";

const LOCALDATA: &str = "data/zipdata.txt";

const BASEURL : &str =  "https://api.weather.gov/points/";

const ALERTURL : &str = "https://api.weather.gov/alerts/active/zone/";


// Curl callback
fn webcall(url: &str, data: &mut Vec<u8>) {
    let mut handle = Easy::new();
    handle
        .useragent(AGENT)
        .expect("ERROR : Could not set useragent...");

    handle.url(url).unwrap();

    let mut transfer = handle.transfer();
transfer
        .write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })
        .unwrap();
    transfer.perform().unwrap();
}


// Stores GPS informatoin
pub struct Pos {
    is_valid: bool,
    pub lat: String,
    pub lon: String,
    pub county : String,
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Latitude : {} | Longitude : {}", self.lat, self.lon)
    }
}


// 
pub struct Cache {
    data: Vec<u8>,
    length: usize,
    pub last_offset: usize,
}

impl Cache {
    pub fn get(&self, index: usize) -> u8 {
        if index < self.length {
            return self.data[index];
        } else {
            return 0;
        }
    }

    pub fn get_data_pointer(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    pub fn set_length(&mut self) {
        self.length = self.data.len();
    }
}

// simple get input call
fn get_input(input: &mut String) {
    let _ = stdout().flush();
    stdin()
        .read_line(input)
        .expect("Did not enter a valid string");
    if let Some('\n') = input.chars().next_back() {
        input.pop();
    }
    if let Some('\r') = input.chars().next_back() {
        input.pop();
    }
}
// minumum 5 char input
fn input_to_zip(input: &str) -> [u8; 5] {
    let mut buf = [0; 5];
    if input.len() < 5 {
        return buf;
    }

    let chars: Vec<char> = input.chars().collect();

    for i in 0..5 {
        buf[i] = chars[i] as u8;
    }

    return buf;
}
// load file into "cache"
fn load_cache(path: &str) -> Cache {
    let mut file = File::open(path).expect(path);

    let mut cache: Cache = Cache {
        data: vec![],
        length: 0,
        last_offset: 0,
    };

    file.read_to_end(cache.get_data_pointer())
        .expect("ERROR : Could not allocate enough RAM for CACHE");
    cache.set_length();


    drop(file);

    return cache;
}


// parse cached data for infomation
fn zip_to_gps(zip: &str, cache: &Cache) -> Pos {
    //let mut file = File::open(LOCALDATA).expect(LOCALDATA);
    //let mut buf : Vec<u8> = vec![];
    //
    let mut buf : [u8; 5];
    let mut offset : usize;

    let mut lat : [char;9] = [' ';9];
    let mut long : [char;11] = [' ';11];

    let scanner = input_to_zip(&zip);
    // zip scanner is fixed lenght
    let scanner_size = 5;

    for i in 10..cache.length - scanner_size {
        //print!("{}", cache.get(i) as char);
        buf = [
            cache.get(i),
            cache.get(i + 1),
            cache.get(i + 2),
            cache.get(i + 3),
            cache.get(i + 4),
        ];

        if scanner == buf && cache.get(i-1) == 10 &&cache.get(i+5) == b',' {
            //println!("ZIP FOUND AT : {}", i);
            offset = i+6;
            for j in 0..5 {
                print!("{}", buf[j] as char);
    
            }
            println!("");

            for j in 0..6{
               lat[j] = cache.get(offset+j) as char; 
            }

            offset = offset + 10;

            for j in 0..8{
               long[j] = cache.get(offset+j) as char; 
            }

            break;
        }
        //println!("{}",i);
    }


    //println!("{:?}",lat);
    //println!("{:?}",long);

    

    let valid : bool;
    if lat[0] != ' '{
        valid = true
        
    } else {
        valid = false
    }

    if lat[5] == '0'{
        lat[5] = '1';
    }
    
    if long[7] == '0'{
       long[7] = '1';
    }
    
    let slat : String = lat.iter().collect();
    let slon : String = long.iter().collect();


    return Pos{
        is_valid : valid,
        lat : slat.trim().to_string(),
        lon : slon.trim().to_string(),
        county : String::from(""),
    }
}

fn printlocal(data : &Vec<u8>){

    println!("#LOCATION#");


    let city = [b'c',b'i',b't',b'y'];
    let mut highscan : [u8;4];
    let state = [b's',b't',b'a',b't',b'e'];
    let mut lowscan : [u8;5];
    
    let mut highprint : bool = false;
    let mut lowprint : bool = false;

    for i in data.len()/2..data.len()-5{
        
        highscan = [data[i],data[i+1],data[i+2],data[i+3]];
        lowscan = [data[i],data[i+1],data[i+2],data[i+3],data[i+4]];

        if highscan == city{
            highprint = true;
        }


        if lowscan == state{
            lowprint = true;
        }

        if highprint {
            if data[i] == b','{
                highprint = false;
                println!("");

            } else if data[i] != b'"'{
                print!("{}",data[i] as char);
            }
        }



        if lowprint {
            if data[i] == b','{
                //lowprint = false;
                println!("");
                break;

            } else if data[i] != b'"'{
                print!("{}",data[i] as char);
            }
        }
        
    }
}

fn find_between_in_data(string : &str, stop_byte : u8, data : &Vec<u8>, skip_into : usize) -> String{
    
    let mut scan_match : Vec<u8> = vec![];
    let mut scan_buf : Vec<u8> = vec![];

    let mut output_buffer : Vec<char> = vec![];


    for c in string.chars(){
        scan_match.push(c as u8);
        scan_buf.push(0);
    }

    let mut index : usize = skip_into;
    let mut not_found : bool = true;
    
    'parse : loop{
        if not_found{
            if index == data.len()-scan_buf.len(){
                break 'parse;
            }
            for j in 0..scan_buf.len(){
                scan_buf[j] = data[j+index];
            }

            if scan_buf == scan_match{
                not_found = false;
                index = index+scan_buf.len()+3;

            }
        } else {

            'filler : loop{
                if data[index] == stop_byte{

                    break 'filler;
                }
                output_buffer.push(data[index] as char);
                index = index + 1;
            }
            break 'parse;
        }


        index = index + 1;

    }

    drop(scan_buf);
    drop(scan_match);

    return output_buffer.iter().collect();


}

fn find_county(data : &Vec<u8>) -> String{
    let find : [char;6] = ['c','o','u','n','t','y'];
    let mut output : [char;6] = [' ';6];
    let mut scanner : [char;6];
    
    let mut index : usize = data.len()/2;


    'parse : loop {

        if index >= data.len()-6{
            break 'parse;
        }

        scanner = [data[index] as char ,data[index+1] as char ,data[index+2] as char ,data[index+3] as char ,data[index+4] as char ,data[index+5] as char ];

        if scanner == find {
            index = index + 47;
            
            for i in 0..6 {
                output[i] = data[index+i] as char;
            }

            break 'parse;

        }

        index = index + 1;

    }


    return output.iter().collect();
    


}


// Only exist to move the matching info for finding links outside of main
fn get_option(option : u8, data : &Vec<u8>) -> String{
    let find : &str;
    let stop : u8;

    match option{
        1 =>{
            find = "forecast";
            stop = b'"';
        },
        2 =>{
            find = "forecastHourly";
            stop = b'"';
        },
        _ => {
            find ="/";
            stop = 10;
        }
    }

    return find_between_in_data(find, stop, data, 1700);

}




struct ForecastData{
    pub number : Vec<u8>,
    pub name : Vec<u8>,
    pub temp : Vec<u8>,
    pub temp_unit : Vec<u8>,
    pub precip_chance : Vec<u8>,
    pub wind_speed : Vec<u8>,
    pub wind_dir : Vec<u8>,
    pub short_forecast : Vec<u8>,
    pub detailed_forcast :  Vec<u8>

}

impl ForecastData {
    pub fn new() -> ForecastData{
        ForecastData{
            number :vec![],
            name : vec![],
            temp : vec![],
            temp_unit : vec![],
            precip_chance : vec![],
            wind_speed : vec![],
            wind_dir : vec![],
            short_forecast : vec![],
            detailed_forcast :vec![]
        }

    }

    pub fn simple_display(&self){

        for i in 0..self.number.len(){
            print!("{}",self.number[i] as char);
        }
        print!(" | ");
        for i in 0..self.name.len(){
            print!("{}",self.name[i] as char);
        }
        print!(" | ");

        for i in 0..self.temp.len(){
            print!("{}",self.temp[i] as char);
        }
        for i in 0..self.temp_unit.len(){
            print!("{}",self.temp_unit[i] as char);
        }

        print!(" | ");

        for i in 0..self.precip_chance.len(){
            print!("{}",self.precip_chance[i] as char);
        }
        print!("% | ");

        for i in 0..self.wind_speed.len(){
            print!("{}",self.wind_speed[i] as char);
        }


        print!(" - ");


        for i in 0..self.wind_dir.len(){
            print!("{}",self.wind_dir[i] as char);
        }

        print!(" | ");

        for i in 0..self.short_forecast.len(){
            print!("{}",self.short_forecast[i] as char);
        }

        println!("");

    }

    pub fn detailed_display(&self){

        for i in 0..self.number.len(){
            print!("{}",self.number[i] as char);
        }
        print!(" | ");
        for i in 0..self.name.len(){
            print!("{}",self.name[i] as char);
        }
        println!("");

        print!(" Temperature : ");
        for i in 0..self.temp.len(){
            print!("{}",self.temp[i] as char);
        }
        for i in 0..self.temp_unit.len(){
            print!("{}",self.temp_unit[i] as char);
        }

        print!(" | Precipitation Chance");


        for i in 0..self.precip_chance.len(){
            print!("{}",self.precip_chance[i] as char);
        }
        print!("% | Wind Info ");

        for i in 0..self.wind_speed.len(){
            print!("{}",self.wind_speed[i] as char);
        }


        print!(" - ");


        for i in 0..self.wind_dir.len(){
            print!("{}",self.wind_dir[i] as char);
        }


        println!("");

        print!("Forecast : ");

        for i in 0..self.detailed_forcast.len() {
            print!("{}",self.detailed_forcast[i] as char);
        }

        println!("");

    }
}




fn parse_forcast(data : &Vec<u8>) -> Vec<ForecastData>{
    
    if data.len() < 10000 {
        println!("ERROR : Forcast data may be invalid");
        println!("ERROR : Log Option unavalible... No log will be made...");
        return vec![];
    }
    
    let mut index = 0;
    //let mut f_count = 0;

    let mut scanner : [u8;6];

    let base_scan = [b'n',b'u',b'm',b'b',b'e',b'r'];
    let temp_scan = [b't',b'e',b'm',b'p',b'e',b'r'];
    let mut temp_not_found : bool = true;
    let precip_scan = [b'v',b'a',b'l',b'u',b'e',b'"'];
    let short_scan = [b's',b'h',b'o',b'r',b't',b'F'];



    let mut forecasts : Vec<ForecastData> = vec![];
    let mut forecast_count = 0;
    
    'initial_parse : loop{
        if index == data.len()-6{
            break 'initial_parse;
        }


        scanner = [data[index],data[index+1],data[index+2],data[index+3],data[index+4],data[index+5]];

        
        if scanner == base_scan{
            forecast_count = forecasts.len();
            forecasts.push(ForecastData::new());
            temp_not_found = true;

            index = index + 8;
            
            //println!("FINDING : number");

            while data[index] != b','{
                forecasts[forecast_count].number.push(data[index]);
                index = index +1;
            }

            index = index + 27;

            //println!("FINDING : name");
            while data[index] != b'"'{
                //print!("{}",data[index]);
                forecasts[forecast_count].name.push(data[index]);
                index = index +1;
            }

            //println!(" ADDED TO {}",forecast_count);

            
        }

        if scanner == temp_scan && temp_not_found{
            temp_not_found = false;
            index = index + 13;

            //println!("FINDING : temp");
            while data[index] != b','{
                forecasts[forecast_count].temp.push(data[index]);
                index = index +1;
            }

            index = index + 38;

            //println!("FINDING : temp unit");
            while data[index] != b'"'{
                forecasts[forecast_count].temp_unit.push(data[index]);
                index = index +1;
            }

        }


        if scanner == precip_scan && !temp_not_found{

            //println!("FINDING : precip chance");
            index = index + 7;
            while data[index] != 10{
                forecasts[forecast_count].precip_chance.push(data[index]);
                index = index + 1;
            }

            index = index + 50;

            //println!("FINDING : wind speed");
            while data[index] != b'"'{
                forecasts[forecast_count].wind_speed.push(data[index]);
                index = index +1;
            }
            index = index + 37;

            //println!("FINDING : wind dir");
            while data[index] != b'"'{
                forecasts[forecast_count].wind_dir.push(data[index]);
                index = index +1;
            }

        }

        if scanner == short_scan{
            index = index + 17;

           // println!("FINDING : forecast summery");
            while data[index] != b'"'{
                forecasts[forecast_count].short_forecast.push(data[index]);
                index = index +1;
            }
            
            index = index + 40;

           // println!("FINDING : detailed forecast");
            while data[index] != b'"'{
                forecasts[forecast_count].detailed_forcast.push(data[index]);
                index = index +1;
            }

        }


        index = index + 1;

    }

    



    println!("COLLECTED DATA : {}",forecasts.len());
    return forecasts;


}


fn display_forecast(data : &Vec<ForecastData>){
    let mut detail : String = String::from("1");
    let mut buf : Vec<char> = vec![];
    
    let mut check_value : String;


    'display : loop {

        for d in data{
            for i in 0..d.number.len(){
                buf.push(d.number[i] as char);
            }
            check_value = buf.iter().collect();
            //println!("< {} | {} >", check_value,detail);
            if check_value.trim() == detail{
                println!("");
                d.detailed_display();
                println!("");
                buf.clear();
                
            } else {

                d.simple_display();
                //println!("");
                buf.clear();

            }
            
            //println!(" VALUE CHECK : {}",detail);
            if detail == "B" || detail == "b"{
                break 'display;
            }


        }


        println!("Options :");
        println!("[NUMBER] : Detailed information for that entry");
        println!("[B]ack");
        print!("[]> ");
        detail.clear();
        get_input(&mut detail);

    }

}


struct AlertData{
    pub number : u8,
    pub urgency : Vec<char>,
    pub event : Vec<char>,
    pub sender : Vec<char>,
    pub sender_name : Vec<char>,
    pub headline : Vec<char>,
    pub description : Vec<char>,
    pub instruction : Vec<char>,
}

impl AlertData{
    pub fn new() -> AlertData{
        AlertData { 
            number : 0,
            urgency : vec![],
            event : vec![],
            sender : vec![],
            sender_name : vec![],
            headline : vec![],
            description : vec![],
            instruction : vec![],
        }
    }


    pub fn detailed_display(&self){
        let mut flag : bool = false;

        println!("{}", "#".repeat(80));

        for i in 0..self.headline.len(){

            if self.headline[i] == 'n' && flag{
                println!("");
                flag = false;
            } else {

                if self.headline[i] == 92 as char{
                    flag = true;
                } else {
                    print!("{}",self.headline[i]); 
                    flag = false;
                }

            }

        }

        println!("");

        for i in 0..self.description.len(){
            if self.description[i] == 'n' && flag{
                println!("");
                flag = false;
            } else {

                if self.description[i] == 92 as char{
                    flag = true;
                } else {
                    print!("{}",self.description[i]); 
                    flag = false;
                }

            }
        }

        println!("");


        for i in 0..self.instruction.len(){
            if self.instruction[i] == 'n' && flag{
                println!("");
                flag = false;
            } else {

                if self.instruction[i] == 92 as char{
                    flag = true;
                } else {
                    print!("{}",self.instruction[i]); 
                    flag = false;
                }

            }


        }

        println!("\n{}", "#".repeat(80));
    }

    pub fn simple_display(&self){

        print!("{} | ",self.number);

        for i in 0..self.event.len(){
            print!("{}",self.event[i]);
        }

        print!(" - ");

        for i in 0..self.urgency.len(){
            print!("{}",self.urgency[i]);
        }

        print!(" | ");

        for i in 0..self.sender_name.len(){
            print!("{}",self.sender_name[i]);
        }


        println!("");
        

    }

}


fn parse_alerts(data : &Vec<u8>) -> Vec<AlertData>{
    let mut output : Vec<AlertData> = vec![];
    let mut out_len = 0;

    let find : [char;6] = ['u','r','g','e','n','c'];
    let mut scanner : [char;6];
    
    let mut index : usize = 0;

    'parse : loop{
        if index >= data.len()-6{
            break 'parse;
        }

        scanner = [data[index] as char ,data[index+1] as char,data[index+2] as char,data[index+3] as char,data[index+4] as char,data[index+5] as char];

        if scanner == find {
            output.push(AlertData::new());
            index = index + 11;

            output[out_len].number = out_len as u8;

            while data[index] != b'"'{
                output[out_len].urgency.push(data[index] as char);
                index = index + 1;
            }
            index = index + 29;


            while data[index] != b'"'{
                output[out_len].event.push(data[index] as char);
                index = index + 1;
            }
            index = index + 30;


            while data[index] != b'"'{
                output[out_len].sender.push(data[index] as char);
                index = index + 1;
            }
            index = index + 34;

            while data[index] != b'"'{
                output[out_len].sender_name.push(data[index] as char);
                index = index + 1;
            }
            index = index + 32;



            while data[index] != b'"'{
                output[out_len].headline.push(data[index] as char);
                index = index + 1;
            }

            index = index + 35;


            while data[index] != b'"'{
                output[out_len].description.push(data[index] as char);
                index = index + 1;
            }


            index = index + 35;


            while data[index] != b'"'{
                output[out_len].instruction.push(data[index] as char);
                index = index + 1;
            }


            out_len = output.len(); 
        }

        index = index +1;
    }


    return output;
}

fn display_alerts(alerts : &Vec<AlertData>){
    let mut detail : String = String::from("S");
    let mut buf : Vec<char> = vec![];
    'display : loop {

        for a in alerts{

            //println!("< {} | {} >", check_value,detail);
            match detail.parse::<u8>(){
                Ok(num) => {
                    if a.number == num {
                        a.detailed_display();
                    }

                },
                Err(_) => {

                    if detail == "S" || detail == "s"{
                        a.simple_display();
                    }


                }

            }
            
            //println!(" VALUE CHECK : {}",detail);


        }

        if detail == "B" || detail == "b"{
            break 'display;
        }

        println!("Options :");
        println!("[NUMBER] : Detailed information for that entry");
        println!("[S]imple Display");
        println!("[B]ack");
        print!("[]> ");
        detail.clear();
        get_input(&mut detail);

    }
}

fn main() {

    // Lazy attempt to find data files
    if !std::path::Path::new(LOCALDATA).exists(){
        let mut ex_path = env::current_exe().expect("ERROR : Could not find working DIR");
        ex_path.pop();
        env::set_current_dir(ex_path).expect("ERROR : Could not change DIR");
    }


    let cache = load_cache(LOCALDATA);
    // Stores location after lookup
    let mut location: Pos;

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        let mut s = String::new();
        print!("Zip Code : ");
        get_input(&mut s);
        location = zip_to_gps(&s, &cache);
    } else {
        location = zip_to_gps(&args[1], &cache);
    }

    println!("{}", location);

    //let base_url = "https://api.weather.gov/points/{latitude},{longitude}";
    let mut url = format!("{BASEURL}{0},{1}",location.lat, location.lon);

    let mut furl : String;
    let mut hurl : String;
    
    let mut web_data : Vec<u8> = Vec::new();
    //let mut fweb_data : Vec<u8> = Vec::new();
    let mut a_web_data : Vec<u8> = Vec::new();

    webcall(&url,&mut web_data);

    location.county = find_county(&web_data);

    printlocal(&web_data);
    println!("county : {}",location.county);


    let mut forecast_data : Vec<ForecastData> = vec![];
    let mut alert_data : Vec<AlertData> = vec![];

    

    let mut user_input = String::new();
    'ui: loop  {
        println!("Please select an option");
        println!("[A]lerts");
        println!("[C]hange Locaton");
        println!("[F]orcast");
        println!("[G]et Data");
        println!("[Q]uit");

        print!("[]> ");

        get_input(&mut user_input);

        
        for c in user_input.chars(){
            match c{
                'F'|'f' => {
                    println!("DISPLAY FORCAST");
                    if forecast_data.len() < 1 {
                        furl = get_option(1, &web_data);
                        println!("TESTING | {}",furl);
                        webcall(&furl,&mut web_data);
                        forecast_data = parse_forcast(&web_data);
                    }
                    display_forecast(&forecast_data);
                }

                'A'|'a' => {
                    println!("Alerts");
                    url = format!("{ALERTURL}{0}",location.county);
                    a_web_data.clear();
                    webcall(&url, &mut a_web_data);
                    alert_data = parse_alerts(&a_web_data);

                    display_alerts(&alert_data);

                    

                }

                'C'|'c' => {
                    web_data.clear();
                    forecast_data.clear();

                    let mut s = String::new();
                    print!("Zip Code : ");
                    get_input(&mut s);
                    location = zip_to_gps(&s, &cache);
                    url = format!("{BASEURL}{0},{1}",location.lat, location.lon);
                    webcall(&url,&mut web_data);
                    location.county = find_county(&web_data);
                    printlocal(&web_data);
                    println!("county : {}",location.county);
                }

                'G'|'g' => {
                    println!("GATHERING AND PARSING DATA...");
                    //stage 1
                    web_data.clear();
                    url = format!("{BASEURL}{0},{1}",location.lat, location.lon);
                    println!("{}",url);
                    webcall(&url,&mut web_data);
                    /*
                    for i in 0..web_data.len(){
                        print!("{}",web_data[i] as char);
                    }
                    println!("");
                    */
                    printlocal(&web_data);
                    // stage 2
                    furl = get_option(1, &web_data);
                    println!("{}",furl);
                    webcall(&furl,&mut web_data);
                    forecast_data = parse_forcast(&web_data);

                    // stage 3
                    /*
                    web_data.clear();
                    webcall(&url,&mut web_data);

                    hurl = get_option(2, &web_data);
                    println!("{}",hurl);
                    webcall(&hurl,&mut web_data);
                    hourly_data = parse_forcast(&web_data);
                    */



                }

                'Q'|'q'=> {
                    break 'ui;
                }
                _ => {}
            }

        }

        user_input.clear();


    }

    println!("");
   
}
