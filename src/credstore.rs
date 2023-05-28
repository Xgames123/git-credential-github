use keyring::Error;
use keyring::Entry;


fn store(username: &str, password : &str){
 let entry = keyring::Entry::new("gh-login", "github.com").expect("Failed get entry in credentials store");

let mut creds = String::new();

creds.push_str(username);
creds.push('/');
creds.push_str(password);

entry.set_password(&creds).expect("Failed to store access key");

}

fn get(){

}