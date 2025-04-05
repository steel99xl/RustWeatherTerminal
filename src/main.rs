use std::env;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io::{Write, stdin, stdout};
use std::time;

use curl::easy::Easy;


const AGENT: &str = "rwt/1.0.0";

const LOCALDATA: &str = "data/zipdata.txt";

const BASEURL : &str =  "https://api.weather.gov/points/";

const ALERTURL : &str = "https://api.weather.gov/alerts/active/zone/";


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

// Curl callback
fn webcall(url: &str, data: &mut Vec<u8>) -> bool {
    let mut input : String = String::from("");
    let mut handle = Easy::new();
    handle
        .useragent(AGENT)
        .expect("ERROR : Could not set useragent...");

    match handle.url(url){
        Ok(_) =>{
            let mut transfer = handle.transfer();
            match transfer.write_function(|new_data| {data.extend_from_slice(new_data);Ok(new_data.len())}) {
                Ok(_) => {
                    match transfer.perform() {
                        Ok(_) => {
                            return true;
                        },
                        Err(e) => {
                            println!("{}",e);
                        }
                    }
                }
                Err(e) => {
                    println!("{}",e);
                }
            }

        },
        Err(e) =>{
            println!("{}",e);

        }
    }



    print!("Re Attempt? [N/y] : ");
    get_input(&mut input);

    if input =="Y" || input == "y" {
        webcall(url,data);
    }


    return false;
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
pub struct Cache<T> {
    data: Vec<T>,
    length: usize,
    pub last_offset: usize,
    birth : time::SystemTime,
    age_limit : time::Duration
}

impl <T>Cache<T> {
    pub fn new(age_limit_in_minutes : u64 ) -> Self{
        Cache{
            data : Vec::new(),
            length : 0,
            last_offset : 0,
            birth : time::SystemTime::now(),
            age_limit : time::Duration::new(age_limit_in_minutes * 60,0),

        }

    }
    pub fn add(&mut self, data : T){
        self.data.push(data);
        self.length = self.length + 1;
    }
    pub fn get(&self, index: usize) -> &T {
        return &self.data[index];

    }

    pub fn clear(&mut self){

        for _ in 0..self.length{
            self.data.pop();
        }

        self.length = 0;
    }

    pub fn set(&mut self, data : Vec<T>){
        self.clear();
        self.data = data;
        self.length = self.data.len();
        self.birth = time::SystemTime::now();
    }

    pub fn get_data_pointer(&self) -> &Vec<T> {
        &self.data
    }

    pub fn get_mut_data_pointer(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    pub fn set_length(&mut self) {
        self.length = self.data.len();
    }

    pub fn is_outdated(&self) -> bool{
        if self.birth + self.age_limit <= time::SystemTime::now(){
            return true;
        } else {
            return false;
        }
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
fn load_cache(path: &str) -> Cache<u8> {
    let mut file = File::open(path).expect(path);

    let mut cache: Cache<u8> = Cache::new(1);

    file.read_to_end(cache.get_mut_data_pointer())
        .expect("ERROR : Could not allocate enough RAM for CACHE");
    cache.set_length();


    drop(file);

    return cache;
}


// parse cached data for infomation
fn zip_to_gps(zip: &str, cache: &Cache<u8>) -> Pos {
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
            *cache.get(i),
            *cache.get(i + 1),
            *cache.get(i + 2),
            *cache.get(i + 3),
            *cache.get(i + 4),
        ];

        if scanner == buf && *cache.get(i-1) == 10 && *cache.get(i+5) == b',' {
            //println!("ZIP FOUND AT : {}", i);
            offset = i+6;
            for j in 0..5 {
                print!("{}", buf[j] as char);

            }
            println!("");

            for j in 0..6{
               lat[j] = *cache.get(offset+j) as char;
            }

            offset = offset + 10;

            for j in 0..8{
               long[j] = *cache.get(offset+j) as char;
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

    if data.len() <= 10 {
        println!("ERROR : NO DATA FOR LOCATION...");
        return;
    }


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

    if data.len() <= 10 {
        return String::from("ERROR : NO DATA");
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
    

    if data.len() <= 10 {
        return String::from("ERROR : NO DATA");
    }

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





    //println!("COLLECTED DATA : {}",forecasts.len());
    return forecasts;


}


fn display_forecast(data : &Vec<ForecastData>){
    let mut detail : String = String::from("1");
    let mut buf : Vec<char> = vec![];

    let mut check_value : String;


    'display : loop {
        if detail == "B" || detail == "b"{
            break 'display;
        }

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

        }


        println!("Options |");
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
    'display : loop {

        if detail == "B" || detail == "b"{
            break 'display;
        }

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

        println!("Options |");
        println!("[NUMBER] : Detailed information for that entry");
        println!("[S]imple Display");
        println!("[B]ack");
        print!("[]> ");
        detail.clear();
        get_input(&mut detail);

    }
}



pub struct ObservationData{
    pub id : Vec<char>,
    pub name : Vec<char>

}

impl ObservationData {
    pub fn new() ->ObservationData{
        ObservationData{
            id : vec![],
            name : vec![]
        }
    }

    pub fn detailed_display(&self){
       let url = format!{"https://api.weather.gov/stations/{0}/observations/latest",self.id.iter().collect::<String>()};
       let mut web_buf : Vec<u8> = vec![];

       let mut name_buf : Vec<char> = vec![];

       let mut unit_buf : Vec<char> = vec![];
       let mut value_buf : Vec<char> = vec![];

       if webcall(&url,&mut web_buf) {
           let mut index : usize = 0;

           let mut find : [char;3] = ['a','g','e'];


           //println!("DISPLAYING : {}",web_buf.len());
           //println!("{}",url);


           'parse : loop{

               if index >= web_buf.len()-3{
                   break 'parse;
               }
               // Finds message
               //
               if find == [web_buf[index] as char, web_buf[index+1] as char, web_buf[index+2] as char]{

                   index = index + 7;
                   while web_buf[index] as char != '"'{
                       print!("{}",web_buf[index] as char);
                       index = index + 1;
                   }
                   println!("");

                   'finder : loop{
                       find = [':',' ','{'];
                       if index >= web_buf.len()-3{
                           break 'finder;
                           //break 'fianl_parse;
                       }

                       if find == [web_buf[index] as char, web_buf[index+1] as char, web_buf[index+2] as char]{
                           name_buf.clear();

                           index = index - 2;


                           while web_buf[index] as char != '"' {
                               name_buf.push(web_buf[index] as char);
                               index = index -1;
                           }

                           if name_buf == ['e','s','a','b']{
                               break 'parse;
                           }

                           for i in (0..name_buf.len()).rev() {
                               print!("{}",name_buf[i]);
                           }

                           print! (" : ");


                           find = ['i','t',':'];
                           // Gets unit and value
                           'reorder : loop {
                               if index >= web_buf.len()-6{
                                   break 'reorder;
                                   //break 'fianl_parse;
                               }
                               if find == [web_buf[index] as char, web_buf[index+1] as char, web_buf[index+2] as char]{
                                   unit_buf.clear();
                                   value_buf.clear();

                                   index = index + 3;

                                   while web_buf[index] as char  != '"'{
                                       unit_buf.push(web_buf[index] as char);
                                       index = index +1;
                                   }

                                   index = index + 24;

                                   while web_buf[index] as char != ',' && web_buf[index] != 10{
                                       value_buf.push(web_buf[index] as char);
                                       index = index +1;

                                   }

                                   for i in 0..value_buf.len(){
                                       print!("{}",value_buf[i]);
                                   }

                                   print!(" ");

                                   for i in 0..unit_buf.len(){
                                       print!("{}",unit_buf[i]);
                                   }

                                   println!("");

                                   break 'reorder;


                               }

                               index = index +1;
                           }

                       }
                       index = index +1;
                   }


               }



               index = index +1;

           }

       }

       drop(name_buf);
       drop(unit_buf);
       drop(value_buf);
       drop(web_buf);
       drop(url);
    }
}



fn find_observation_stations(data : &Vec<u8>, skip : usize) -> Vec<ObservationData>{
    let mut url : Vec<char> = vec![];
    let mut find  : [char;3] = ['o','n','s'];
    let mut index : usize = skip;

    'parse : loop{
        if index >= data.len() - 3{
            break 'parse;
        }

        if find == [data[index] as char,data[index+1] as char,data[index+2] as char]{
            index = index + 7;
            while data[index] as char != '"'{
                url.push(data[index] as char);
                index = index + 1;
            }

            break 'parse;
        }

        index = index + 1;
    }


    let mut web_buffer : Vec<u8> = vec![];
    let mut output : Vec<ObservationData> = vec![];
    let mut output_len : usize;


    //output.iter().collect();
    let tmp : String = url.iter().collect();
    webcall(&tmp, &mut web_buffer);


    index = 0;
    find = ['i','e','r'];


    'parse : loop{
        if index >= web_buffer.len() - 3{
            break 'parse;
        }

        if find == [web_buffer[index] as char,web_buffer[index+1] as char,web_buffer[index+2] as char]{
            output.push(ObservationData::new());
            output_len = output.len() - 1;
            index = index + 7;

            while web_buffer[index] as char != '"'{
                output[output_len].id.push(web_buffer[index] as char);
                index = index + 1;
            }
            index = index + 28;

            while web_buffer[index] as char != '"'{
                output[output_len].name.push(web_buffer[index] as char);
                index = index + 1;
            }
        }

        index = index + 1;

    }



    drop(url);
    //drop(find);
    //drop(index);
    return output;


}

fn display_observations(observations : &Vec<ObservationData>){
   let mut input : String = String::from("S");
   let mut buf : Vec<char> = vec![];
   let mut count = 0 ;
   let char_limit = 60;

   let mut station_found = false;

   'display : loop {

       if input =="S" || input == "s" {
           station_found = false;
       }
       if input == "B" || input == "b"{
           break 'display;
       }

       buf.clear();

       for i in input.trim().chars(){
           buf.push(i.to_ascii_uppercase());
       }


       for o in observations {
           //println!("{:?} | {:?}",buf,o.id);
           if buf == o.id{
               station_found = true;
               o.detailed_display();
               break;
           }

       }

       if !station_found{
           for o in observations{
               if count >= char_limit {
                   count = 0;
                   println!("");
               }
               print!("<ID: {} | {} >", o.id.iter().collect::<String>(),o.name.iter().collect::<String>());
               print!(" ");
               count = count + o.id.len() + o.name.len() + 12;

           }
       }


        println!("");

        println!("Options |");
        println!("[ ID ] : View Observation From Station");
        println!("[S]imple Display");
        println!("[B]ack");
        print!("[]> ");
        input.clear();
        get_input(&mut input);
   }

}




fn weather(){

    let zip_cache = load_cache(LOCALDATA);
    // Stores location after lookup
    let mut location: Pos;

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        let mut s = String::new();
        print!("Zip Code : ");
        get_input(&mut s);
        location = zip_to_gps(&s, &zip_cache);
    } else {
        location = zip_to_gps(&args[1], &zip_cache);
    }



    //let base_url = "https://api.weather.gov/points/{latitude},{longitude}";
    let mut url = format!("{BASEURL}{0},{1}",location.lat, location.lon);
    let mut unit : String = String::from("us");
    let mut furl : String;


    let mut web_data : Vec<u8> = Vec::new();
    let mut f_web_data : Vec<u8> = Vec::new();
    let mut a_web_data : Vec<u8> = Vec::new();

    webcall(&url,&mut web_data);

    location.county = find_county(&web_data);
    //ourl = find_observation_stations(&web_data,1700);

    printlocal(&web_data);


    let mut forecast_cache : Cache<ForecastData> = Cache::new(15);
    let mut alert_cache : Cache<AlertData> = Cache::new(5);
    let mut observation_cache : Cache<ObservationData> = Cache::new(5);


    let mut user_input = String::new();
    'ui: loop  {
        if !location.is_valid{
            println!("WARNING : Location is not valid, attampting to get weather will fail catastrophically!!!")
        }

        println!("Please select an option");
        println!("[A]lerts");
        println!("[F]orecast");
        println!("[L]ocaton");
        println!("[O]bservations");
        println!("[Q]uit");
        //println!("[R]efresh Forecast");
        println!("[U]nits");

        print!("[]> ");

        get_input(&mut user_input);


        for c in user_input.chars(){
            match c{
                'F'|'f' => {
                    if forecast_cache.length < 1 || forecast_cache.is_outdated(){
                        //println!("{} | {}", forecast_cache.length, forecast_cache.is_outdated());
                        println!("UPDADING FORECAST CACHE");

                        f_web_data.clear();

                        furl = get_option(1, &web_data);
                        furl = format!{"{furl}?units={unit}"};

                        if webcall(&furl,&mut f_web_data) {
                            forecast_cache.set(parse_forcast(&f_web_data));
                        }
                    }
                    println!("DISPLAY FORECAST");
                    display_forecast(forecast_cache.get_data_pointer());
                }

                'A'|'a' => {
                    if alert_cache.length < 1 || alert_cache.is_outdated(){
                        println!("UPDATING ALERT CACHE");
                        a_web_data.clear();
                        if webcall(&format!("{ALERTURL}{0}",location.county), &mut a_web_data){
                            alert_cache.set(parse_alerts(&a_web_data));
                        }
                    }
                    println!("DISPLAY ALERTS");

                    display_alerts(alert_cache.get_data_pointer());



                }

                'L'|'l' => {
                    web_data.clear();
                    a_web_data.clear();
                    f_web_data.clear();
                    forecast_cache.clear();
                    alert_cache.clear();
                    observation_cache.clear();

                    let mut s = String::new();
                    print!("Zip Code : ");
                    get_input(&mut s);
                    location = zip_to_gps(&s, &zip_cache);
                    url = format!("{BASEURL}{0},{1}",location.lat, location.lon);
                    if webcall(&url,&mut web_data){
                        location.county = find_county(&web_data);
                        printlocal(&web_data);
                        println!("county : {}",location.county);
                    }

                    drop(s);
                }

                'O'|'o' => {

                    if observation_cache.length < 1 || observation_cache.is_outdated(){
                        println!("UPDATING OBSERVATION SATION LIST CACHE");
                        web_data.clear();
                        if webcall(&url,&mut web_data){
                            observation_cache.set(find_observation_stations(&web_data,web_data.len()/2));
                        }
                    }

                    println!("DISPLAY OBSERVATION STATION LIST");
                    display_observations(observation_cache.get_data_pointer());


                }

                'U'|'u' => {
                    println!("SELECT UNIT TYPE |");
                    println!("[I]merial");
                    println!("[M]etric");
                    print!("[]> ");
                    let mut tinput = String::new();
                    get_input(&mut tinput);
                    for t in tinput.chars() {
                        match t{
                            'I'|'i' =>{
                                unit = String::from("us");
                                println!("UNIT TYPE : Imperial");
                            },
                            'M'|'m' =>{
                                unit = String::from("si");
                                println!("UNIT TYPE : Metric");
                            },
                            _ => {
                                break;
                            }
                        }
                    }
                    forecast_cache.clear();

                    drop(tinput);

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

fn main() {

    // Lazy attempt to find data files
    if !std::path::Path::new(LOCALDATA).exists(){
        let mut ex_path = env::current_exe().expect("ERROR : Could not find working DIR");
        ex_path.pop();
        env::set_current_dir(ex_path).expect("ERROR : Could not change DIR");
    }



    weather();


}
