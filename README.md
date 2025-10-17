# genpdf-json

rckive-genpdf is a user-friendly PDF generator written in pure Rust.

genpdf-json uses rckive-genpdf and generates a PDF from JSON data.

The library can be used in two ways:

1. Search for a JSON file and create a new PDF file in the specified path.
2. Pass a JSON string and receive a PDF file in Base64 text.


```rust
use genpdf_json;
let _ = genpdf_json::render_json_file("file.json", "report.pdf");
```

The json file must have config and elements:

```
{
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
}
```

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






