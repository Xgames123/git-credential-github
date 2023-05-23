mod params;

pub struct Param {
	name: &str
	value: &str
	
}

pub fn Param::new(key: &str, value : &str){
	Param{name: key value:value}
}

pub fn Param::from_string(s &str) -> Result<Param, ParamParserError>{
	
	
}


pub struct Params{
	
	hashmap: HashMap<String, Param>
	
	pub fn get(key: &str) -> Option<String>{
		return hashmap.get(string);
	}
	
	pub fn add(key: String, value : String){
		
	}
	
}


#[derive(Debug, Clone)]
struct ParamParserError{
	data : &str
}

pub fn Params::from_string(s : &str) -> Result<Params, ParamParserError>{
	
	fn split_param(param: &str) -> Result<(&str, &str), ParamParserError> {
    for (i, &character) in param.as_bytes().iter().enumerate() {
        if character == b'=' {
            return Ok((&param[..i], &param[(i + 1)..]));
        }
    }
    return Err(ParamParserError {param=param});
}
