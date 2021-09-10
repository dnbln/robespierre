macro_rules! book_chapter {
    ($path_in_book_src:literal) => {
        concat!("../../../book/src/", $path_in_book_src)
    };
}

macro_rules! book_chapter_test {
    ($chapter_name:ident, $file_name:literal) => {
        mod $chapter_name {
            doc_comment::doctest!(book_chapter!($file_name));
        }
    };
}

book_chapter_test!(writing_an_example_bot, "writing-an-example-bot.md");
book_chapter_test!(framework, "framework.md");
book_chapter_test!(user_data, "user-data.md");
book_chapter_test!(extractors, "extractors.md");
book_chapter_test!(writing_custom_frommessage_impls, "writing-custom-FromMessage-impls.md");
book_chapter_test!(permissions, "permissions.md");
