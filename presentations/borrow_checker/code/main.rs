
fn longest<'a>(a: &'a String, b: &'a String) -> &'a String {
  if a.len() > b.len() { a } else { b }
}


fn main() {

  let string1 = String::from("this is a very very long string");  
  let result;                                                     
  {                                                               
    let string2 = String::from("Short");                          
    result = longest(&string1, &string2);                         
  }                                                               
  println!("{result}");                                           

}