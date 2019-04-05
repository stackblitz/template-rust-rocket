#[macro_use] extern crate rocket;

#[derive(UriDisplayQuery)]
struct Foo1;
//~^ ERROR not supported

#[derive(UriDisplayQuery)]
struct Foo2();
//~^ ERROR not supported

#[derive(UriDisplayQuery)]
enum Foo3 { }
//~^ ERROR not supported

#[derive(UriDisplayQuery)]
enum Foo4 {
    Variant,
    //~^ ERROR not supported
}

#[derive(UriDisplayQuery)]
struct Foo5(String, String);
//~^ ERROR exactly one

#[derive(UriDisplayQuery)]
struct Foo6 {
    #[form(field = 123)]
    //~^ ERROR invalid value: expected string
    field: String,
}

#[derive(UriDisplayPath)]
struct Foo7(String, usize);
//~^ ERROR exactly one

#[derive(UriDisplayPath)]
struct Foo8;
//~^ ERROR exactly one

#[derive(UriDisplayPath)]
enum Foo9 {  }
//~^ ERROR not supported

#[derive(UriDisplayPath)]
struct Foo10 {
//~^ ERROR not supported
    named: usize
}
