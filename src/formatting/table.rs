use std::fmt::Display;

use crate::config;

/// Ascii table display for data
pub struct AsciiTable<'a, T>
where
    T: Display,
{
    rows: Vec<Vec<T>>,
    seperate_cols: bool,
    seperate_rows: bool,
    config: &'a config::TableConfig,
}

impl<T> AsciiTable<'_, T>
where
    T: Display,
{
    /// Creates an AsciiTable
    pub fn new(rows: Vec<Vec<T>>, config: &config::TableConfig) -> AsciiTable<T> {
        AsciiTable {
            rows,
            seperate_cols: true,
            seperate_rows: false,
            config,
        }
    }

    /// Sets whether rows will be seperated by a line
    pub fn seperate_rows(mut self, enable: bool) -> Self {
        self.seperate_rows = enable;
        self
    }

    /// Sets whether columns will be seperated by a line
    pub fn seperate_cols(mut self, enable: bool) -> Self {
        self.seperate_cols = enable;
        self
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

impl<T> Display for AsciiTable<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let col_lengths = AsciiTable::calc_col_lengths(&self.rows);

        let col_seperator = if self.seperate_cols {
            format!("{} ", self.config.vertical)
        } else {
            " ".to_string()
        };

        let row_seperator = if self.seperate_rows {
            let cross_string = if self.seperate_cols {
                format!(
                    "{}{}",
                    self.config.vertical_horizontal, self.config.horizontal
                )
            } else {
                self.config.horizontal.to_string().repeat(2)
            };

            format!(
                "\n{}\n",
                col_lengths
                    .clone()
                    .into_iter()
                    .map(|length| self.config.horizontal.to_string().repeat(length + 1))
                    .collect::<Vec<String>>()
                    .join(&cross_string)
            )
        } else {
            "\n".to_string()
        };

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
                    .join(&col_seperator) // Join columns of strings into single string
            })
            .collect::<Vec<String>>()
            .join(&row_seperator); // Join rows of strings into single string

        write!(f, "{table_str}")
    }
}
