pub struct Table<R> {
    columns: Vec<Column<R>>,
}

pub struct Column<R> {
    select: Box<dyn Fn(&R) -> String>,
    name: String,
}

impl<R> Table<R> {
    pub fn new(columns: Vec<Column<R>>) -> Self {
        Table { columns }
    }

    pub fn print(&self, data: &[R]) {
        let s_data = self.select(data);

        let widths = self.calc_widths(&s_data);

        let c_names: Vec<String> = self
            .columns
            .iter()
            .map(|c| &c.name)
            .zip(&widths)
            .map(|(r, w)| format!("{:<width$}", r, width = w))
            .collect();
        println!("| {} |", c_names.join(" | "));

        let seps: Vec<String> = widths.iter()
            .map(|w| format!("{:-<width$}", "", width = w + 2))
            .collect();
        println!("|{}|", seps.join("+"));

        for r in s_data.iter() {
            let row: Vec<String> = r
                .iter()
                .zip(&widths)
                .map(|(r, w)| format!("{:<width$}", r, width = w))
                .collect();
            println!("| {} |", row.join(" | "));
        }
    }

    fn select(&self, data: &[R]) -> Vec<Vec<String>> {
        data.iter()
            .map(|x| self.columns.iter().map(|c| c.select(x)).collect())
            .collect()
    }

    fn calc_widths(&self, s_data: &Vec<Vec<String>>) -> Vec<usize> {
        let w: Vec<usize> = self.columns.iter().map(|c| c.name.len()).collect();

        self.columns
            .iter()
            .enumerate()
            .map(|(i, _)| {
                s_data
                    .iter()
                    .fold(w[i], |a, x| std::cmp::max(x[i].len(), a))
            })
            .collect()
    }
}

impl<R> Column<R> {
    pub fn new<F>(name: &str, f: F) -> Self
    where
        F: Fn(&R) -> String + 'static,
    {
        Column {
            select: Box::new(f),
            name: name.to_string(),
        }
    }

    pub fn select(&self, x: &R) -> String {
        (self.select)(x)
    }
}

#[cfg(test)]
mod test {

    use super::{Column, Table};

    struct Foo<'a> {
        a: i32,
        b: &'a str,
    }

    #[test]
    fn test_table() {
        let table = Table::new(vec![
            Column::<Foo>::new("Foo", |x| x.a.to_string()),
            Column::<Foo>::new("B", |x| x.b.to_string()),
        ]);

        let data = vec![
            Foo { a: 12, b: "foo" },
            Foo { a: 32, b: "barber" },
            Foo { a: 53, b: "bin" },
        ];

        table.print(&data);
    }
}
