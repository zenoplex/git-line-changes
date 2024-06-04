use std::io::{stdout, BufWriter, Write};

#[derive(Debug)]
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(headers: Vec<String>, rows: Vec<Vec<String>>) -> Table {
        Table { headers, rows }
    }

    fn column_widths(&self) -> Vec<usize> {
        let mut widths = vec![0; self.headers.len()];

        // Check header widths
        for (i, header) in self.headers.iter().enumerate() {
            widths[i] = header.len();
        }

        // Check row widths
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if cell.len() > widths[i] {
                    widths[i] = cell.len();
                }
            }
        }

        widths
    }

    pub fn render(&self) {
        let stdout = stdout();
        let mut handle = BufWriter::new(stdout);

        let widths = self.column_widths();

        for (i, header) in self.headers.iter().enumerate() {
            write!(handle, "{:width$} ", header, width = widths[i]).unwrap();
        }

        writeln!(handle).unwrap();

        for width in &widths {
            write!(handle, "{:-<width$} ", "-", width = width).unwrap();
        }

        writeln!(handle).unwrap();

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                write!(handle, "{:width$} ", cell, width = widths[i]).unwrap();
            }
            writeln!(handle).unwrap();
        }
    }
}
