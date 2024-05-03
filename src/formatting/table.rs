use std::fmt::Display;

/// Ascii table display for data
pub struct AsciiTable<T>
where
    T: Display,
{
    rows: Vec<Vec<T>>,
    seperate_cols: bool,
    seperate_rows: bool,
    box_chars: BoxChars,
}

impl<T> AsciiTable<T>
where
    T: Display,
{
    /// Creates an AsciiTable
    pub fn from(rows: Vec<Vec<T>>) -> AsciiTable<T> {
        AsciiTable {
            rows,
            seperate_cols: true,
            seperate_rows: false,
            box_chars: BoxChars::new(
                HorChars {
                    hor: '─',
                    up_hor: '┴',
                    down_hor: '┬',
                },
                VertChars {
                    vert: '│',
                    vert_right: '├',
                    vert_left: '┤',
                },
                CornerChars {
                    down_right: '┌',
                    down_left: '┐',
                    up_right: '└',
                    up_left: '┘',
                },
                '┼',
            ),
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

impl<T> Display for AsciiTable<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let col_lengths = AsciiTable::calc_col_lengths(&self.rows);

        let col_seperator = if self.seperate_cols {
            format!("{} ", self.box_chars.vert)
        } else {
            " ".to_string()
        };

        let row_seperator = if self.seperate_rows {
            let cross_string = if self.seperate_cols {
                format!("{}{}", self.box_chars.vert_hor, self.box_chars.hor)
            } else {
                self.box_chars.hor.to_string().repeat(2)
            };

            format!(
                "\n{}\n",
                col_lengths
                    .clone()
                    .into_iter()
                    .map(|length| self.box_chars.hor.to_string().repeat(length + 1))
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

/// Helper struct for utf-8 box drawing
#[allow(dead_code)]
struct BoxChars {
    /// Char: ─
    hor: char,
    /// Char: │
    vert: char,
    /// Char: ┴
    up_hor: char,
    /// Char: ┬
    down_hor: char,
    /// Char: ├
    vert_right: char,
    /// Char: ┤
    vert_left: char,
    /// Char: ┼
    vert_hor: char,
    /// Char: ┌
    down_right: char,
    /// Char: ┐
    down_left: char,
    /// Char: └
    up_right: char,
    /// Char: ┘
    up_left: char,
}

impl BoxChars {
    fn new(
        hor_chars: HorChars,
        vert_chars: VertChars,
        corners: CornerChars,
        vert_hor: char,
    ) -> BoxChars {
        BoxChars {
            hor: hor_chars.hor,
            vert: vert_chars.vert,
            up_hor: hor_chars.up_hor,
            down_hor: hor_chars.down_hor,
            vert_right: vert_chars.vert_right,
            vert_left: vert_chars.vert_left,
            vert_hor,
            down_right: corners.down_right,
            down_left: corners.down_left,
            up_right: corners.up_right,
            up_left: corners.up_left,
        }
    }
}

struct HorChars {
    /// Char: │
    hor: char,
    /// Char: ┴
    up_hor: char,
    /// Char: ┬
    down_hor: char,
}

struct VertChars {
    /// Char: ─
    vert: char,
    /// Char: ├
    vert_right: char,
    /// Char: ┤
    vert_left: char,
}

struct CornerChars {
    /// Char: ┌
    down_right: char,
    /// Char: ┐
    down_left: char,
    /// Char: └
    up_right: char,
    /// Char: ┘
    up_left: char,
}
