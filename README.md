# genpdf-json

Fast PDF generator

rckive-genpdf is a user-friendly PDF generator written in pure Rust.

genpdf-json uses rckive-genpdf (https://github.com/RCKIVE/rckive-genpdf-rs) and generates a PDF from JSON data.

The library can be used in two ways:

1. Search for a JSON file and create a new PDF file in the specified path.
2. Pass a JSON string and receive a PDF file in Base64 text.

El JSON puede ser pasado directamente o usando un archivo sqlite.db


Options:
```rust

let _ = genpdf_json::render_json_file("file.json", "report.pdf");

let file_pdf_base64 = genpdf_json::render_json_base64(&json_string);

let _ = genpdf_json::render_file_from_sqlite("file.db", "report.pdf");

let file_pdf_base64 = genpdf_json::render_base64_from_sqlite("file.db");

```

The structure of the document is:
```
    {
        "config": {...},
        "elements": [...elements...]
    }
```

examples:

```rust
use genpdf_json;
let _ = genpdf_json::render_json_file("file.json", "report.pdf");
```

file.json:

```
{
    "config": {
        "title": "report genpdf", 
        "style": {}, 
        "page_size": "A4", 
        "fonts": [], 
        "default_font": {"font_family_name": "LiberationSans", "dir": "/usr/share/fonts/truetype/liberation"}, 
        "head_page": {"type": "paragraph", "value": [{"text": "report genpdf-rs", "bold": true, "size": 8, "italic": true}], "alignment": "right"}, 
        "margins": [7, 10, 10, 10], "line_spacing": 1.0,
        "skip_warning_overflowed": true
        }, 
    "elements": [
                    {
                        "type": "layout", 
                        "orientation": "vertical", 
                        "elements": [
                                        {
                                            "type": "paragraph", 
                                            "value": [{"text": "Invoice", "bold": true, "size": 25}], 
                                            "alignment": "center"}, 
                                        {"type": "break", "value": 1}
                                    ], 
                        "frame": {"thickness": 0.1}, 
                        "padding": [1, 1, 1, 1]
                    }, 
                    {"type": "break", "value": 2}, 
                    {
                        "type": "paragraph", 
                        "value": [{"text": "details", "bold": true, "size": 12}], 
                        "alignment": "center"
                    }                
            ]
}
```


Using base64 string:
```rust
use serde_json::json;
use genpdf_json;
fn main() {
    let data_value = json!({
        "config": {
            "title": "report genpdf", 
            "style": {}, 
            "page_size": "A4", 
            "fonts": [], 
            "default_font": {"font_family_name": "LiberationSans", "dir": "/usr/share/fonts/truetype/liberation"}, 
            "head_page": {"type": "paragraph", "value": [{"text": "report genpdf-rs", "bold": true, "size": 8, "italic": true}], "alignment": "right"}, 
            "margins": [7, 10, 10, 10], "line_spacing": 1.0
            }, 
        "elements": [
                        {
                            "type": "layout", 
                            "orientation": "vertical", 
                            "elements": [
                                            {
                                                "type": "paragraph", 
                                                "value": [{"text": "Invoice", "bold": true, "size": 25}], 
                                                "alignment": "center"}, 
                                            {"type": "break", "value": 1}
                                        ], 
                            "frame": {"thickness": 0.1}, 
                            "padding": [1, 1, 1, 1]
                        }, 
                        {"type": "break", "value": 2}, 
                        {
                            "type": "paragraph", 
                            "value": [{"text": "details", "bold": true, "size": 12}], 
                            "alignment": "center"
                        }                
                ]
    });

    let json_string: String = data_value.to_string();
    
    let file_pdf_base64 = genpdf_json::render_json_base64(&json_string);
    println!("{:?}",file_pdf_base64);
}
```

SQLite:
To save the config:
In a table called config, use a column named data (Text), and in this column, save a record with the config's JSON.
For the elements:
In a table called elements, use the id (autoincrement) column and the element (Text) column, and save the elements in each record in the JSON order.
Optionally, for table layouts, save the rows in a new table separate from config and elements, with the structure id (autoincrement) and row (Text). Then, reassign the value to the JSON "rows" with the name of the table created.


Type support
___________

config:
```
"config":{
    "title":"", 
    "style": style, 
    "page_size": string or [float, float]  -> "A4", "Legal", "Letter", or [200,200]
    "fonts" : [{"font_family_name":"", dir:""}],
    "default_font": {"font_family_name":"", dir:""}
    "line_spacing": float,
    "margins": [float, float, float, float],
    "head_page": paragraph, or [paragraph, paragraph, paragraph] maximum 3 paragraphs
    "footer_page": [paragraph, paragraph, paragraph] maximum 3 paragraphs
    "head_page_count": paragraph,
    "header_frame": line_style,
    "footer_frame": line_style,
    "page_frame": line_style,
    "deafault_font_size": int,
    "skip_warning_overflowed": bool -> Skip the page size exceeded warning when the paragraph exceeds the layout
}
```

line_style:
```
 {
    "thickness":float,
    "color":color,
    "dash": int, "gap": int, "dash2": int, "gap2": int,
    "top": bool, "right": bool, "bottom": bool, "left": bool (only frame),
    "background": bool, (only frame)
    "background_color": color, (only frame)
 }
 
```

margins:
```
    [top, right, bottom, left]  float or int
```

style:
```
 {
    "bold":bool,
    "italic":bool,
    "font_family_name": string,
    "color": color,
    "line_spacing": float,
    "size": int,
    "fit_size_to": int -> auto size to minimum
 }
```

alignment:
```
    "left"
    "center"
    "right"
```

string_style:
```
 {
    "text": string
    "bold":bool,
    "italic":bool,
    "font_family_name": string,
    "color": color,
    "line_spacing": float,
    "size": int,
    "fit_size_to": int -> auto size to minimum
 }
```

color:
```
    {"type":"rgb", "value":[int, int, int]} 0 - 255
    {"type":"cmyk", "value":[int, int, int, int]}
    {"type":"greyscale", "value":int}
```
elements
```
 {
    "type": layout,
    "orientation":"vertical"
    "frame": line_style,
    "style": style,
    "padding": margins,
    "orphan": bool,
    "position": [float, float],
    "elements": [...elements...]
 }
 
 {
    "type": layout,
    "orientation":"horizontal"
    "column_weights": [], array of integers
    "frame": line_style,
    "style": style,
    "padding": margins,
    "elements": [...elements column_weights.len()...] -> 
 }
 
 {
    "type": table_layout,
    "frame_decorator":[[inner(bool), outer(bool), cont(bool)], line_style]
    "column_weights": [], array of integers
    "frame": line_style,
    "style": style,
    "padding": margins,
    "rows": [...rows column_weights.len()...] -> 
 }
 
 {
    "type": unordered_list,
    "frame": line_style,
    "style": style,
    "padding": margins,
    "elements": [...elements...]
    "bullet": string
 }
 
 {
    "type": ordered_list,
    "frame": line_style,
    "style": style,
    "padding": margins,
    "elements": [...elements...]
    "start": int
 }
 
 {
    "type": paragraph,
    "frame": line_style,
    "style": style,
    "padding": margins,
    "value": [...string_style...]
    "alignment" alignment
    "bullet": string
 }
 
 {
    "type": text,
    "value": string_style,
    "orphan": bool,
    "position": [float, float]
 }
 
 {
    "type": image,
    "path": string,
    "base64": string,
    "frame": line_style,    
    "padding": margins,    
    "alignment" alignment
    "position": [float, float],
    "scale": float,
    "rotation", float  +-
    "dpi": float    
 }
 
 {
    "type" : "break", 
    "value": float, Negative values ​​are allowed
 }
 
 {  
    "type" : "page_break"
 } 
```

Other important information:
If you need a small PDF file size on disk, use light fonts, as they are embedded within the PDF.

Links:

https://github.com/numaelis/genpdf-json-bin

https://gitlab.com/numaelis/pygenpdf_json

https://gitlab.com/numaelis/pygenpdf

