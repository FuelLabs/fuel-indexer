extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

#[indexer(
    manifest = "examples/nest/nest.manifest.yaml"
)]
pub mod nest_index_mod {

    fn nest_handler(_block_data: BlockData) {
        Logger::info("Doing a thing,");

        let genre1 = Genre {
            id: 1,
            name: "horror".to_string(),
        };
        let genre2 = Genre {
            id: 2,
            name: "mystery".to_string(),
        };
        let genre3 = Genre {
            id: 3,
            name: "business".to_string(),
        };

        genre1.save();
        genre2.save();
        genre3.save();

        let person1 = Person {
            id: 1,
            name: "Ava".to_string(),
        };
        let person2 = Person {
            id: 2,
            name: "Noel".to_string(),
        };
        let person3 = Person {
            id: 3,
            name: "Julio".to_string(),
        };

        person1.save();
        person2.save();
        person3.save();

        let city1 = City {
            id: 1,
            name: "Los Angeles".to_string(),
        };
        let city2 = City {
            id: 2,
            name: "New York".to_string(),
        };
        let city3 = City {
            id: 3,
            name: "Raleigh".to_string(),
        };

        city1.save();
        city2.save();
        city3.save();

        let author1 = Author {
            id: 1,
            name: "Brian".to_string(),
            genre: genre1.id,
        };
        let author2 = Author {
            id: 2,
            name: "James".to_string(),
            genre: genre2.id,
        };
        let author3 = Author {
            id: 3,
            name: "Susan".to_string(),
            genre: genre3.id,
        };

        author1.save();
        author2.save();
        author3.save();

        let library1 = Library {
            id: 1,
            name: "Scholar Library".to_string(),
            city: city1.id,
        };
        let library2 = Library {
            id: 2,
            name: "Ronoke Library".to_string(),
            city: city2.id,
        };
        let library3 = Library {
            id: 1,
            name: "Scholastic Library".to_string(),
            city: city3.id,
        };

        library1.save();
        library2.save();
        library3.save();

        let book1 = Book {
            id: 1,
            name: "Gone with the wind".to_string(),
            author: author1.id,
            library: library1.id,
            genre: genre1.id,
        };
        let book2 = Book {
            id: 2,
            name: "Othello".to_string(),
            author: author2.id,
            library: library2.id,
            genre: genre2.id,
        };
        let book3 = Book {
            id: 3,
            name: "Cyberpunk 2021".to_string(),
            author: author3.id,
            library: library3.id,
            genre: genre3.id,
        };

        book1.save();
        book2.save();
        book3.save();

        let bookclub1 = BookClub {
            id: 1,
            book: book1.id,
            member: person1.id,
        };
        let bookclub2 = BookClub {
            id: 2,
            book: book2.id,
            member: person2.id,
        };
        let bookclub3 = BookClub {
            id: 3,
            book: book3.id,
            member: person3.id,
        };

        bookclub1.save();
        bookclub2.save();
        bookclub3.save();
    }
}
