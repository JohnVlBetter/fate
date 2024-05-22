use syn::{parse::{Parse, ParseStream}, token::Comma, Ident, LitInt};

// 描述宏需要解析到的数据
struct AllTuples {
    macro_ident: Ident,
    start: usize,
    end: usize,
    ident: Ident,
}

impl Parse for AllTuples {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_ident = input.parse::<Ident>()?; // 解析使用哪个宏定义
        input.parse::<Comma>()?;// 解析逗号
        let start = input.parse::<LitInt>()?.base10_parse()?; // 解析开始数字
        input.parse::<Comma>()?;// 解析逗号
        let end = input.parse::<LitInt>()?.base10_parse()?;// 解析结束数字
        input.parse::<Comma>()?;// 解析逗号
        let ident =input.parse::<Ident>()?; // 解析使用什么去定义泛型参数

        Ok(AllTuples {
            macro_ident,
            start,
            end,
            ident,
        })
    }
}

pub fn all_tuples(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AllTuples);
    let len = input.end - input.start;
    let mut ident_tuples = Vec::with_capacity(len);
    for i in input.start..=input.end {
        let ident = format_ident!("{}{}", input.ident, i);
        ident_tuples.push(quote! {
            #ident
        });
    }

    let macro_ident = &input.macro_ident;
    let invocations = (input.start..=input.end).map(|i| {
        let ident_tuples = &ident_tuples[..i];
        quote! {
            #macro_ident!(#(#ident_tuples),*);
        }
    });
    TokenStream::from(quote! {
        #(
            #invocations
        )*
    })
}

macro_rules! impl_tuple_system_param {
    ($($param: ident),*) => {
        impl<$($param: SystemParam,)*> SystemParam
            for ($($param,)*)
        {
        }
    };
}

all_tuples!(impl_system_param_functions, 0, 15, P);

pub trait SystemParamFunction<Param>: 'static {
    fn run(&mut self, params: Param);
}

impl<F, Param> System for F
where
    F: SystemParamFunction<Param>,
{
    fn run(&mut self) {
        self.run(todo!())
    }
}

pub trait SystemParam {}

impl SystemParam for u32 {}
impl SystemParam for String {}
impl SystemParam for bool {}

pub trait System {
    fn run(&mut self);
}

pub struct App {
    systems: Vec<Box<dyn System>>,
}

impl App {
    pub fn new() -> Self {
        Self { systems: vec![] }
    }

    pub fn add_system<T: System + 'static>(&mut self, system: T) -> &mut Self {
        self.systems.push(Box::new(system));
        self
    }

    pub fn run(&mut self) {
        for item in self.systems.iter_mut() {
            item.run();
        }
    }
}

impl<T: FnMut()> System for T {
    fn run(&mut self) {
        self();
    }
}

fn main() {
    App::new()
        .add_system(test_system)
        .add_system(test_system2)
        .run();
}

fn test_system() {
    println!("test_system");
}

fn test_system2() {
    println!("test_system2");
}