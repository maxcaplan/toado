use std::fmt::Display;

/// Ascii table display for data
pub struct AsciiTable<T>
where
    T: Display,
{
    rows: Vec<Vec<T>>,
}

impl<T> AsciiTable<T>
where
    T: Display,
{
    /// Creates an AsciiTable
    pub fn from(rows: Vec<Vec<T>>) -> AsciiTable<T> {
        AsciiTable { rows }
    }

    /// Calculates the length of the longest value in each column of the table.
    /// Returns vector of said values
    fn calc_col_lengths(rows: &[Vec<T>]) -> Vec<usize> {
        let mut rows = rows.iter();
        if let Some(cols) = rows.next() {
            let mut col_lengths: Vec<usize> =
                cols.iter().map(|value| value.to_string().len()).collect();

            for cols in rows {
                for (i, val) in cols.iter().enumerate() {
                    let length = val.to_string().len();

                    if length > col_lengths[i] {
                        col_lengths[i] = length;
                    }
                }
            }

            col_lengths
        } else {
            Vec::new()
        }
    }
}

impl<T> Display for AsciiTable<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let col_lengths = AsciiTable::calc_col_lengths(&self.rows);
        let table_str = self
            .rows
            .iter()
            .map(|col| {
                col.iter()
                    .enumerate()
                    .map(|(i, val)| {
                        let len_dif = col_lengths[i] - val.to_string().len();
                        format!("{val}{}", " ".repeat(len_dif + 1)) // Add padding to value string
                    })
                    .collect::<Vec<String>>()
                    .join("| ") // Join columns of strings into single string
            })
            .collect::<Vec<String>>()
            .join("\n"); // Join rows of strings into single string

        write!(f, "{table_str}")
    }
}
