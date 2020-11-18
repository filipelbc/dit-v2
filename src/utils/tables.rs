pub struct Table<R> {
    columns: Vec<Column<R>>,
}

pub struct Column<R> {
    select: Box<dyn Fn(&R) -> String>,
}

#[macro_export]
macro_rules! table {
    ($($x:expr),+ $(,)?) => {
        Table::new(vec![$(Column::new($x),)+])
    };
}

impl<R> Table<R> {
    pub fn new(columns: Vec<Column<R>>) -> Self {
        Table { columns }
    }

    pub fn print(&self, data: &Vec<R>) {
        let s_data = self.select(data);

        let widths = self.calc_widths(&s_data);

        for r in s_data.iter() {
            let row: Vec<String> = r
                .iter()
                .zip(&widths)
                .map(|(r, w)| format!("{:<width$}", r, width = w))
                .collect();
            println!("| {} |", row.join(" | "));
        }
    }

    fn select(&self, data: &Vec<R>) -> Vec<Vec<String>> {
        data.iter()
            .map(|x| self.columns.iter().map(|c| c.select(x)).collect())
            .collect()
    }

    fn calc_widths(&self, s_data: &Vec<Vec<String>>) -> Vec<usize> {
        self.columns
            .iter()
            .enumerate()
            .map(|(i, _)| s_data.iter().fold(0, |a, x| std::cmp::max(x[i].len(), a)))
            .collect()
    }
}

impl<R> Column<R> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&R) -> String + 'static,
    {
        Column {
            select: Box::new(f),
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
        let table = table![
            |x: &Foo| x.a.to_string(),
            |x: &Foo| x.b.to_string(),
        ];

        let data = vec![
            Foo {
                a: 12,
                b: "foo",
            },
            Foo {
                a: 32,
                b: "barber",
            },
            Foo {
                a: 53,
                b: "bin",
            },
        ];

        table.print(&data);
    }
}
