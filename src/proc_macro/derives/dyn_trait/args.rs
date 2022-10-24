use super::*;

pub
struct Args {
    pub dyn_: Token![dyn],
}

impl Parse for Args {
    fn parse (input: ParseStream<'_>)
      -> Result<Args>
    {
        // We special case the `dyn` parameter. It does not provide any
        // information to the macro invocation; it just makes said invocation
        // more readable. We thus require it once, and at the beginning of the
        // args: the idea is that users could grep for `derive_ReprC(dyn`.
        let dyn_ = match input.parse() {
            | Ok(it) => it,
            | Err(err) => return Err(Error::new(
                err.span(),
                &format!("{err} — usage: `#[derive_ReprC(dyn, …)]`"),
            )),
        };
        let _: Option<Token![,]> = input.parse()?;
        while input.is_empty().not() {
            let snoopy = input.lookahead1();
            match () {
                | _case if input.peek(Token![dyn]) => {
                    return Err(input.error("duplicate arg"));
                },
                | _todo_other_cases if false => {},
                | _default => return Err(snoopy.error()),
            }
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(Self {
            dyn_,
        })
    }
}
