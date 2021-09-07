use crate::framework::standard::extractors::Argument;

use super::{ArgsConfig, ArgsLexer};

#[test]
fn args_lexer() {
    {
        let mut args_lexer = ArgsLexer::new(
            "aaa bbb",
            ArgsConfig {
                delimiters: smallvec::smallvec![" ".into()],
            },
        );

        assert_eq!(args_lexer.next(), Some(Argument::Simple("aaa")));
        assert_eq!(args_lexer.next(), Some(Argument::Simple("bbb")));
        assert_eq!(args_lexer.next(), None);
    }

    {
        let mut args_lexer = ArgsLexer::new(
            "aaa   bbb",
            ArgsConfig {
                delimiters: smallvec::smallvec![" ".into()],
            },
        );

        assert_eq!(args_lexer.next(), Some(Argument::Simple("aaa")));
        assert_eq!(args_lexer.next(), Some(Argument::Empty));
        assert_eq!(args_lexer.next(), Some(Argument::Empty));
        assert_eq!(args_lexer.next(), Some(Argument::Simple("bbb")));
        assert_eq!(args_lexer.next(), None);
    }
}
