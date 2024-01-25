use std::collections::HashMap;

use csv::Reader;
use serde::Deserialize;

fn main() -> anyhow::Result<()> {
    KanaChartBuilder::new()?.build()
}

struct KanaChartBuilder {
    map_kana_by_romaji: HashMap<String, Kana>,
    chart_html: &'static str,
    cell_html: &'static str,
}

impl KanaChartBuilder {
    pub fn new() -> anyhow::Result<Self> {
        let kanas: Result<Vec<Kana>, _> =
            Reader::from_reader(include_bytes!("kana.csv").as_slice())
                .deserialize()
                .collect();
        let map_kana_by_romaji: HashMap<_, _> = kanas?
            .into_iter()
            .map(|kana| (kana.romaji.clone(), kana))
            .collect();
        let this = Self {
            map_kana_by_romaji,
            chart_html: include_str!("chart.html"),
            cell_html: include_str!("cell.html"),
        };
        Ok(this)
    }
    pub fn build(&self) -> anyhow::Result<()> {
        let romaji_cells: Vec<_> = self
            .chart_html
            .lines()
            .filter(|line| line.contains("<td>"))
            .filter(|line| line.contains("</td>"))
            .filter(|line| !line.contains("<td></td>"))
            .map(str::trim)
            .collect();
        let mut output_html = self.chart_html.to_string();
        for romaji_cell in romaji_cells.into_iter() {
            output_html = output_html.replace(romaji_cell, &self.replace_romaji_cell(romaji_cell)?);
        }

        let output_path = "target/output.html";
        std::fs::write(output_path, output_html)?;
        println!(
            "Rendered a webpage at `{}`. Now you can open it with a web browser and print it.",
            output_path
        );
        Ok(())
    }

    fn replace_romaji_cell(&self, romaji_cell: &str) -> anyhow::Result<String> {
        let romaji = romaji_cell.replace("<td>", "").replace("</td>", "");
        let kana = self
            .map_kana_by_romaji
            .get(&romaji)
            .ok_or_else(|| anyhow::anyhow!("Unknown romaji {}", &romaji))?;
        let kanji = if kana.common_source.is_empty() {
            format!(
                r#"
                    <td class="kanji kanji-hiragana">{}</td>
                    <td class="kanji kanji-katakana">{}</td>
                "#,
                kana.hiragana_source, kana.katakana_source
            )
        } else {
            format!(
                r#"<td class="kanji" colspan="2">{}</td>"#,
                kana.common_source
            )
        };
        let mut output_cell = self
            .cell_html
            .replace('@', &kana.romaji)
            .replace("あ", &kana.hiragana)
            .replace("ア", &kana.katakana)
            .replace(r#"<div class="kanji-placeholder" />"#, &kanji);
        if !kana.deprecated {
            output_cell =
                output_cell.replace("chart-cell-table-deprecated", "chart-cell-table-in-use");
        }
        output_cell = format!("<td>{}</td>", output_cell);
        Ok(output_cell)
    }
}

#[derive(Deserialize)]
struct Kana {
    romaji: String,
    hiragana: String,
    hiragana_source: String,
    katakana: String,
    katakana_source: String,
    common_source: String,
    deprecated: bool,
}
