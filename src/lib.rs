use rckive_genpdf::{
    elements, fonts, style, PaperSize, Margins, Alignment, Element, elements::{Paragraph, IntoBoxedElement, Text, PaddedElement, TableLayout, FrameCellDecorator, LinearLayout, OrderedList, UnorderedList, BulletPoint, Break, PageBreak},
};

use rckive_genpdf::fonts::{FontFamily, Font, FontData};

use rckive_genpdf::error::{Error};
use std::path;
use std::fs;

use serde_json::{json, Value};
use std::{collections::HashMap};

use rusqlite::{params, Connection, Result};

type PdfDocument = rckive_genpdf::Document;

struct GenpdfJson {
    doc: rckive_genpdf::Document,
    alignment_map: HashMap<String, Alignment>,    
    font_cache: HashMap<String, FontFamily<Font>>,
    db_path: String,
}

pub trait RootLayout {
    fn push<E: IntoBoxedElement>(&mut self, element: E) -> ();
    fn push_row(&mut self, row: Vec<Box<dyn Element>>) -> Result<(), Error>;
    fn push_cell(&mut self, cell: Box<dyn Element>) -> ();
    fn push_static<E: Element + 'static>(&mut self, element: E);
    fn type_name(&self) -> &'static str;
}

struct LLayout{
    root: elements::LinearLayout,    
}

struct TLayout{
    root: elements::TableLayout,    
}

struct ULayout{
    root: elements::UnorderedList,    
}

struct OLayout{
    root: elements::OrderedList,    
}

struct VecLayout{
    root: Vec<Box<dyn Element>>
}

struct NoneLayout{
    root: i8,    
}

impl RootLayout for LLayout{
    fn push<E: IntoBoxedElement>(&mut self, element: E) {
        self.root.push(element);
    }
    fn push_row(&mut self, _row: Vec<Box<dyn Element>>) -> Result<(), Error> {
        Ok(())
    }
    fn push_static<E: Element + 'static>(&mut self, _element: E){}
    fn push_cell(&mut self, _cell: Box<dyn Element>){}
    fn type_name(&self) -> &'static str {
        "LinearLayout"
    }
}

impl RootLayout for TLayout{
    fn push<E: IntoBoxedElement>(&mut self, _element: E) {        
    }
    fn push_row(&mut self, row: Vec<Box<dyn Element>>) -> Result<(), Error> {
        let _ = self.root.push_row(row);
        Ok(())
    }
    fn push_cell(&mut self, _cell: Box<dyn Element>){}
    fn push_static<E: Element + 'static>(&mut self, _element: E){}
    fn type_name(&self) -> &'static str {
        "TableLayout"
    }
}

impl RootLayout for ULayout{
    fn push<E: IntoBoxedElement>(&mut self, _element: E) {        
    }
    fn push_row(&mut self, _row: Vec<Box<dyn Element>>) -> Result<(), Error> {
        Ok(())
    }
    fn push_cell(&mut self, _cell: Box<dyn Element>){}
    fn push_static<E: Element + 'static>(&mut self, element: E){
        self.root.push(element);
    }
    fn type_name(&self) -> &'static str {
        "StaticList"
    }
}

impl RootLayout for OLayout{
    fn push<E: IntoBoxedElement>(&mut self, _element: E) {        
    }
    fn push_row(&mut self, _row: Vec<Box<dyn Element>>) -> Result<(), Error> {
        Ok(())
    }
    fn push_cell(&mut self, _cell: Box<dyn Element>){}
    fn push_static<E: Element + 'static>(&mut self, element: E){
        self.root.push(element);
    }
    fn type_name(&self) -> &'static str {
        "StaticList"
    }
}

impl RootLayout for VecLayout{
    fn push<E: IntoBoxedElement>(&mut self, _element: E) {        
    }
    fn push_row(&mut self, _row: Vec<Box<dyn Element>>) -> Result<(), Error> {
        Ok(())
    }
    fn push_cell(&mut self, cell: Box<dyn Element>){
        self.root.push(cell);
    }
    fn push_static<E: Element + 'static>(&mut self, _element: E){}
    fn type_name(&self) -> &'static str {
        "VecLayout"
    }
}

impl RootLayout for NoneLayout{
    fn push<E: IntoBoxedElement>(&mut self, _element: E) {        
    }
    fn push_row(&mut self, _row: Vec<Box<dyn Element>>) -> Result<(), Error> {
        Ok(())
    }
    fn push_cell(&mut self, _cell: Box<dyn Element>){}
    fn push_static<E: Element + 'static>(&mut self, _element: E){}
    fn type_name(&self) -> &'static str {
        "NoneLayout"
    }
}

fn get_font_default(json_config: &serde_json::Value) -> Result<FontFamily<FontData>, String> {
    if let Some(default_font) = json_config.get("default_font").and_then(|v| v.as_object()) {            
        if let Some(font_family_name) = default_font.get("font_family_name").and_then(|v| v.as_str()) { 
            if let Some(dir) = default_font.get("dir").and_then(|v| v.as_str()) { 
                let font_family = fonts::from_files(dir, font_family_name, None)
                                    .expect("Failed to load the font family");  
                return Ok(font_family);
            }
        }                            
    }
    Err("error default font".to_string())
    /*let boxed_str: Box<&str> = Box::new("An error occurred");
    let error_message: String = boxed_str.to_string();
    let boxed_error: Box<dyn std::error::Error> = error_message.into();
    Err(boxed_error)*/     
}

fn get_color(val_color: &serde_json::Value) -> style::Color {
    let mut mcolor = style::Color::Rgb(0,0,0);
    if let Some(ctype) = val_color.get("type").and_then(|v| v.as_str()) {
        match ctype {
            "rgb" => {
            if let Some(value) = val_color.get("value").and_then(|v| v.as_array()) {
                if value.len() == 3 {
                    mcolor = style::Color::Rgb(value[0].as_f64().unwrap_or(0.0) as u8, 
                                                    value[1].as_f64().unwrap_or(0.0) as u8, 
                                                    value[2].as_f64().unwrap_or(0.0) as u8);
                    }
                }
            }
            
            "cmyk" => {
                if let Some(value) = val_color.get("value").and_then(|v| v.as_array()) {
                    if value.len() == 4 {
                        mcolor = style::Color::Cmyk(value[0].as_f64().unwrap_or(0.0) as u8, 
                                                        value[1].as_f64().unwrap_or(0.0) as u8, 
                                                        value[2].as_f64().unwrap_or(0.0) as u8,
                                                        value[3].as_f64().unwrap_or(0.0) as u8);
                    }
                }
            }                
            "greyscale" => {
                if let Some(value) = val_color.get("value").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as u8)) {                        
                    mcolor = style::Color::Greyscale(value);                            
                }
            }
            &_ => todo!()
        }
    }
    return mcolor;
}
     
fn get_head_style(val_style: &serde_json::Value, font_cache: &HashMap<String, FontFamily<Font>>) -> style::Style {
    let mut mstyle = style::Style::new();
                                    
    if let Some(bold) = val_style.get("bold").and_then(|v| v.as_bool()) {
        if bold{
            mstyle.set_bold();
        }
    }
    if let Some(italic) = val_style.get("italic").and_then(|v| v.as_bool()) {
        if italic{
            mstyle.set_italic();
        }
    }       
    if let Some(font_family_name) = val_style.get("font_family_name").and_then(|v| v.as_str()) {  
        if let Some(family) = font_cache.get(font_family_name) {                                        
            mstyle.set_font_family(*family);
        }                                                                    
    }
    if let Some(size) = val_style.get("size").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as u8)) {
        mstyle.set_font_size(size);
    }
    if let Some(line_spacing) = val_style.get("line_spacing").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)) {
        mstyle.set_line_spacing(line_spacing);
    }
    
    if let Some(val_color) = val_style.get("color") { 
            let color = get_color(val_color);
            mstyle.set_color(color);            
    }    
    return mstyle;
}
/// GenpdfJson
impl GenpdfJson {
    fn new(json_config: &serde_json::Value, db_path: impl AsRef<path::Path>) -> Self {        
        let mut alignment_map = HashMap::new();
        alignment_map.insert("center".to_string(), Alignment::Center);
        alignment_map.insert("left".to_string(), Alignment::Left);
        alignment_map.insert("right".to_string(), Alignment::Right);                       
        
        let default_font = get_font_default(&json_config).unwrap();
        let mut doc = PdfDocument::new(default_font);
        
        // Skip the page size exceeded warning
        if let Some(skip_warning_overflowed) = json_config.get("skip_warning_overflowed").and_then(|v| v.as_bool()) { 
            doc.set_skip_warning_overflowed(skip_warning_overflowed);
        }
                
        let mut fcache = HashMap::new();
        if let Some(fonts) = json_config.get("fonts").and_then(|v| v.as_array()) {
            for font in fonts {
                if let Some(font_family_name) = font.get("font_family_name").and_then(|v| v.as_str()) { 
                    if let Some(dir) = font.get("dir").and_then(|v| v.as_str()) { 
                        let font_family = fonts::from_files(dir, font_family_name, None)
                                            .expect("Failed to load the font family");
                        let family = doc.add_font_family(font_family);
                        fcache.insert(font_family_name.to_string(), family);                        
                    }
                }                
            }
        }
        
        if let Some(title) = json_config.get("title").and_then(|v| v.as_str()) {            
            doc.set_title(title);
        } 
        doc.set_minimal_conformance();
        let mut config_line_spacing = 1.25;
        if let Some(line_spacing) = json_config.get("line_spacing").and_then(|v| Some(v.as_f64().unwrap() as f32)){
            config_line_spacing = line_spacing
        }
        doc.set_line_spacing(config_line_spacing);  
        
        let mut decorator = rckive_genpdf::SimplePageDecorator::new();
        let mut _top_margin = 0.0;
        let mut _bottom_margin = 0.0;
        let mut _left_margin = 0.0;
        let mut _right_margin = 0.0;
        if let Some(margins) = json_config.get("margins").and_then(|v| v.as_array()){
            let t = margins[0].as_f64().unwrap_or(0.0) as f32;
            let r = margins[1].as_f64().unwrap_or(0.0) as f32;
            let b = margins[2].as_f64().unwrap_or(0.0) as f32;
            let l = margins[3].as_f64().unwrap_or(0.0) as f32;
            decorator.set_margins(Margins::trbl(t,r,b,l));
            _top_margin = t.clone();
            _bottom_margin = b.clone();
            _left_margin = l.clone();
            _right_margin = r.clone();
        }else{
            if let Some(margins) = json_config.get("margins").and_then(|v| Some(v.as_f64().unwrap() as f32)){
                decorator.set_margins(margins);
                _top_margin = margins.clone();
                _bottom_margin = margins.clone();
                _left_margin = margins.clone();
                _right_margin = margins.clone();
            }
        }            
        
        // Allow the use of 3 paragraphs in the page header.
        // It is not possible to dynamically include a layout within the footer or page header because it is necessary to implement `clone` in the layout.
        
        let mut page_count_text = "".to_string();
        let mut page_count_alignment = Alignment::Left;
        let mut page_count_style = style::Style::new();
        
        let mut font_size = 9;
        
        let mut count_head_paragraph = 0;
        // let mut head_page_style = style::Style::new();
                
        let mut head_paragraph_0 = Paragraph::default();
        let mut head_paragraph_1 = Paragraph::default();
        let mut head_paragraph_2 = Paragraph::default();
        
        if let Some(head_page) = json_config.get("head_page").and_then(|v| v.as_array()){
            for para in head_page {
                if count_head_paragraph > 2 { break; }
                if let Some(alignment) = para.get("alignment").and_then(|v| v.as_str()) {
                    if count_head_paragraph == 0 {
                        head_paragraph_0.set_alignment(*alignment_map.get(alignment).unwrap());
                    }
                    if count_head_paragraph == 1 {
                        head_paragraph_1.set_alignment(*alignment_map.get(alignment).unwrap());
                    }
                    if count_head_paragraph == 2 {
                        head_paragraph_2.set_alignment(*alignment_map.get(alignment).unwrap());
                    }                    
                }
                if let Some(value) = para.get("value").and_then(|v| v.as_array()) {
                    for val_style in value {      
                        let head_page_style = get_head_style(&val_style, &fcache);                                
                        if let Some(text) = val_style.get("text").and_then(|v| v.as_str()) {                            
                            if count_head_paragraph == 0 {
                                head_paragraph_0.push_styled(text, head_page_style);
                            }
                            if count_head_paragraph == 1 {
                                head_paragraph_1.push_styled(text, head_page_style);
                            }
                            if count_head_paragraph == 2 {
                                head_paragraph_2.push_styled(text, head_page_style);
                            }
                        }                                
                    }
                }
                count_head_paragraph +=1;
            }
        }else{
            if let Some(head_page) = json_config.get("head_page") {
                count_head_paragraph +=1;
                if let Some(alignment) = head_page.get("alignment").and_then(|v| v.as_str()) {
                    head_paragraph_0.set_alignment(*alignment_map.get(alignment).unwrap());                    
                }             
                if let Some(value) = head_page.get("value").and_then(|v| v.as_array()) {
                    for val_style in value {      
                        let head_page_style = get_head_style(&val_style, &fcache);                                
                        if let Some(text) = val_style.get("text").and_then(|v| v.as_str()) {   
                            head_paragraph_0.push_styled(text, head_page_style);
                        }                                
                    }
                }
            }
        }
                
        if let Some(head_page_count) = json_config.get("head_page_count") {
            if let Some(alignment) = head_page_count.get("alignment").and_then(|v| v.as_str()) {
                page_count_alignment = *alignment_map.get(alignment).unwrap();
            }             
            if let Some(value) = head_page_count.get("value").and_then(|v| v.as_array()) {
                for val_style in value {      
                    page_count_style = get_head_style(&val_style, &fcache);                                
                    if let Some(text) = val_style.get("text").and_then(|v| v.as_str()) {                        
                        page_count_text = text.to_string();
                    }                                
                }
            }
        }       
                
        if let Some(deafault_font_size) = json_config.get("deafault_font_size").and_then(|v| Some(v.as_f64().unwrap_or(9.0) as u8)) { 
            font_size = deafault_font_size;
        }
                
        decorator.set_header( move |page|{
            
            let mut layout = elements::LinearLayout::vertical();
            
            if count_head_paragraph > 0 {  
                let head_paragraph_0 = head_paragraph_0.clone();
                layout.push(head_paragraph_0);
                if count_head_paragraph > 1 {
                    let head_paragraph_1 = head_paragraph_1.clone();
                    layout.push(head_paragraph_1);
                }
                if count_head_paragraph > 2 {
                    let head_paragraph_2 = head_paragraph_2.clone();
                    layout.push(head_paragraph_2);
                }
                layout.push(elements::Break::new(1.));
            }
            if page > 1 && page_count_text!= "".to_string()  {
                layout.push(
                    elements::Paragraph::new(format!("{} {}",page_count_text, page)).aligned(page_count_alignment).styled(page_count_style),
                );
                layout.push(elements::Break::new(1.));
            }
            layout.styled(style::Style::new())
        });
        
        let mut _height = 0.0;
        let mut _widht = 0.0;
        
        doc.set_font_size(font_size);                               
        
        if let Some(page_size) = json_config.get("page_size").and_then(|v| v.as_array()) { 
            if page_size.len()>=2 {
                let width = page_size[0].as_f64().unwrap_or(0.0) as f32;
                let height = page_size[1].as_f64().unwrap_or(0.0) as f32;
                doc.set_paper_size(rckive_genpdf::Size::new(width, height));
                _height = height.clone();
                _widht = width.clone();
            }            
        }else{
            if let Some(page_size) = json_config.get("page_size").and_then(|v| v.as_str()) { 
                match page_size {
                    "A4" => {
                        doc.set_paper_size(PaperSize::A4);
                        _height = 297.0;
                        _widht = 210.0;
                    }
                    "Legal" => {
                        doc.set_paper_size(PaperSize::Legal);
                        _height = 356.0;
                        _widht = 216.0;
                    }
                    "Letter" => {
                        doc.set_paper_size(PaperSize::Letter);
                        _height = 279.0;
                        _widht = 216.0;
                    }
                    _ =>{
                    }
                }
            }
        }
        
        // Allow the use of 3 paragraphs in the footer.
        // It is not possible to dynamically include a layout within the footer or page header because it is necessary to implement `clone` in the layout.
        let x = 0;
        let mut y = _height - _bottom_margin; //_top_margin - 
        
        let mut count_footer_paragraph = 0;       
                
        let mut footer_paragraph_0 = Paragraph::default();
        let mut footer_paragraph_1 = Paragraph::default();
        let mut footer_paragraph_2 = Paragraph::default();
        
        if let Some(footer_page) = json_config.get("footer_page").and_then(|v| v.as_array()){
            for para in footer_page {
                if count_footer_paragraph > 2 { break; }
                if let Some(alignment) = para.get("alignment").and_then(|v| v.as_str()) {
                    if count_footer_paragraph == 0 {
                        footer_paragraph_0.set_alignment(*alignment_map.get(alignment).unwrap());
                    }
                    if count_footer_paragraph == 1 {
                        footer_paragraph_1.set_alignment(*alignment_map.get(alignment).unwrap());
                    }
                    if count_footer_paragraph == 2 {
                        footer_paragraph_2.set_alignment(*alignment_map.get(alignment).unwrap());
                    }                    
                }
                if let Some(value) = para.get("value").and_then(|v| v.as_array()) {
                    for val_style in value {      
                        let footer_page_style = get_head_style(&val_style, &fcache);                                
                        if let Some(text) = val_style.get("text").and_then(|v| v.as_str()) {                            
                            if count_footer_paragraph == 0 {
                                // footer_page_style_0 = footer_page_style.clone();
                                footer_paragraph_0.push_styled(text, footer_page_style);
                            }
                            if count_footer_paragraph == 1 {
                                // footer_page_style_1 = footer_page_style.clone();
                                footer_paragraph_1.push_styled(text, footer_page_style);
                            }
                            if count_footer_paragraph == 2 {
                                // footer_page_style_2 = footer_page_style.clone();
                                footer_paragraph_2.push_styled(text, footer_page_style);
                            }
                        }                                
                    }
                }
                count_footer_paragraph +=1;
            }
        }        
        let width_area =  _widht - _left_margin - _right_margin;                
        if count_footer_paragraph > 0 {  
            let line_off = config_line_spacing.clone();
            let mut hei0 = footer_paragraph_0.get_height(doc.context(), width_area);
            
            if count_footer_paragraph > 1 {
                hei0 += footer_paragraph_1.get_height(doc.context(), width_area);   
                // line_off += config_line_spacing.clone();
            }
            if count_footer_paragraph > 2 {
                hei0 += footer_paragraph_2.get_height(doc.context(), width_area);
            }
            let hei0: f32 = hei0.into();
            y -= hei0;
                        
            let off_bottom_margin = hei0.clone() - line_off;
            decorator.set_margins(Margins::trbl(_top_margin, _right_margin, _bottom_margin + off_bottom_margin, _left_margin));
        }
        
        decorator.set_footer( move |page|{     
            
            let mut layout = elements::LinearLayout::vertical().with_orphan(true);
            
            layout.set_orphan_position(x, y);
            if count_footer_paragraph > 0 && width_area > 10.0{  
                let footer_paragraph_0 = footer_paragraph_0.clone();
                layout.push(footer_paragraph_0);
                if count_footer_paragraph > 1 {
                    let footer_paragraph_1 = footer_paragraph_1.clone();
                    layout.push(footer_paragraph_1);
                }
                if count_footer_paragraph > 2 {
                    let footer_paragraph_2 = footer_paragraph_2.clone();
                    layout.push(footer_paragraph_2);
                }
                layout.push(elements::Break::new(1.));
            }
            // TODO page count in footer
            // if page > 1 && page_count_text!= "".to_string()  {
            //     layout.push(
            //         elements::Paragraph::new(format!("{} {}",page_count_text, page)).aligned(page_count_alignment).styled(page_count_style),
            //     );
            //     layout.push(elements::Break::new(1.));
            // }
            layout.styled(style::Style::new())
        });
        
        doc.set_page_decorator(decorator);
                   
       //Enabling hyphenation helps with word wrapping, but not all words. This needs to be improved.
        use hyphenation::Load;

        doc.set_hyphenator(
            hyphenation::Standard::from_embedded(hyphenation::Language::EnglishUS)
                .expect("Failed to load hyphenation data"),
        );             
        
        let db_path = db_path.as_ref().display().to_string();
        GenpdfJson {
            doc,
            alignment_map,            
            font_cache: fcache,
            db_path,
        }
    }
    
    fn render_json_file(mut self, json_obj: &serde_json::Value, path: impl AsRef<path::Path>) -> Result<(), Box<dyn std::error::Error>> {
        self.push_elements(json_obj)?;
        self.doc.render_to_file(path)?;
        Ok(())
    }
    
    fn render_json_base64(mut self, json_obj: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
        self.push_elements(json_obj)?;
        let bytes = self.doc.render_to_base64()?;
        Ok(bytes)
        
    }
    
    fn push_elements_from_sqlite(&mut self, db_path: impl AsRef<path::Path>) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;
        let mut stmt = conn.prepare("SELECT element FROM elements ORDER BY id ASC")?;
        let mut rows = stmt.query(params![])?;
        while let Some(row) = rows.next()? {            
            let data_string: String = row.get("element")?;
            let json_value: Value = serde_json::from_str(&data_string)?;
            let mut none_ = NoneLayout{root:0};
            self.push_element(&json_value, &mut none_)?;
        }
        Ok(())
    }
    
    fn render_file_from_sqlite(mut self, db_path: impl AsRef<path::Path>, path: impl AsRef<path::Path>) -> Result<(), Box<dyn std::error::Error>> {        
        self.push_elements_from_sqlite(db_path)?;               
        self.doc.render_to_file(path)?;
        Ok(())
    }
    
    fn render_base64_from_sqlite(mut self, db_path: impl AsRef<path::Path>) -> Result<String, Box<dyn std::error::Error>> {
        self.push_elements_from_sqlite(db_path)?;
        let bytes = self.doc.render_to_base64()?;
        Ok(bytes)        
    }
    
    fn push_elements(&mut self, json_obj: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(elements) = json_obj.get("elements").and_then(|v| v.as_array()) {
            for element in elements {                
                let mut none_ = NoneLayout{root:0};
                self.push_element(element, &mut none_)?;
            }
        }
        Ok(())
    }   
    
    fn extra_push(&mut self, json_obj: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(elements) = json_obj.get("extra_elements").and_then(|v| v.as_array()) {
            let mut _layout = LinearLayout::vertical();
            let mut layout = LLayout{root:_layout};
            for element in elements {                
                self.push_element(element, &mut layout)?;
            }
            self.doc.extra_push(layout.root);
        }
        Ok(())
    }   
    
    fn get_paddings(&self, item_obj: &serde_json::Value) -> Vec<f32> {
        let mut paddings: Vec<f32> = vec![0.1,0.1,0.1,0.1];
        if let Some(padding) = item_obj.get("padding").and_then(|v| v.as_array()) {
            if padding.len()>=4{
                let t = padding[0].as_f64().unwrap_or(0.0) as f32;
                let r = padding[1].as_f64().unwrap_or(0.0) as f32;
                let b = padding[2].as_f64().unwrap_or(0.0) as f32;
                let l = padding[3].as_f64().unwrap_or(0.0) as f32;
                paddings.clear();
                paddings.push(t);
                paddings.push(r);
                paddings.push(b);
                paddings.push(l);                                                                    
            }
        }else{
            if let Some(one_padding) = item_obj.get("padding").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)) {                            
                    paddings.clear();
                    paddings.push(one_padding);
                    paddings.push(one_padding);
                    paddings.push(one_padding);
                    paddings.push(one_padding);                                                                                                
            }
        }
        paddings
    }
    
    fn get_dashes(&self, frame: &serde_json::Map<std::string::String, serde_json::Value>) -> (i64, i64, i64, i64) {
        let (mut frame_dash, mut frame_gap, mut frame_dash2, mut frame_gap2) = (0, 0, 0, 0);        
        if let Some(dash) = frame.get("dash").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as i64)){
            frame_dash = dash;
        }
        if let Some(gap) = frame.get("gap").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as i64)){
            frame_gap = gap;
        }
        if let Some(dash2) = frame.get("dash2").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as i64)){
            frame_dash2 = dash2;
        }
        if let Some(gap2) = frame.get("gap2").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as i64)){
            frame_gap2 = gap2;
        }
        return (frame_dash, frame_gap, frame_dash2, frame_gap2);
        
    }
    fn get_sides(&self, frame: &serde_json::Map<std::string::String, serde_json::Value>) -> (bool, bool, bool, bool) {
        let (mut ftop, mut fright, mut fbottom, mut fleft) = (true, true, true, true);
        if let Some(top) = frame.get("top").and_then(|v| v.as_bool()){
            ftop = top;
        }
        if let Some(right) = frame.get("right").and_then(|v| v.as_bool()){
            fright = right;
        }
        if let Some(bottom) = frame.get("bottom").and_then(|v| v.as_bool()){
            fbottom = bottom;
        }
        if let Some(left) = frame.get("left").and_then(|v| v.as_bool()){
            fleft = left;
        }
        return (ftop, fright, fbottom, fleft);
        
    }
    fn get_style(&self, val_style: &serde_json::Value) -> style::Style {
        let mut mstyle = style::Style::new();
                                      
        if let Some(bold) = val_style.get("bold").and_then(|v| v.as_bool()) {
            if bold{
                mstyle.set_bold();
            }
        }
        if let Some(italic) = val_style.get("italic").and_then(|v| v.as_bool()) {
            if italic{
                mstyle.set_italic();
            }
        }
        if let Some(font_family_name) = val_style.get("font_family_name").and_then(|v| v.as_str()) {  
            if let Some(family) = self.font_cache.get(font_family_name) {                                        
                mstyle.set_font_family(*family);
            }                                                                    
        }
        if let Some(size) = val_style.get("size").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as u8)) {
            mstyle.set_font_size(size);
        }
        if let Some(fit_font_size_to) = val_style.get("fit_size_to").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as u8)) {
            mstyle.set_fit_font_size_to(fit_font_size_to);
        }
        if let Some(line_spacing) = val_style.get("line_spacing").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)) {
            mstyle.set_line_spacing(line_spacing);
        }        
        if let Some(val_color) = val_style.get("color") {       //.and_then(|v| v.as_object())
                let color = get_color(val_color);
                mstyle.set_color(color);            
        }    
        return mstyle;
    }
    
    fn match_text_paragraph<T: RootLayout, U: rckive_genpdf::Element + 'static>(&mut self,  element:U, root_layout: &mut T, 
                                                                                has_frame: bool, frame_thickness: f32, 
                                                                                frame_color: style::Color, frame_dash: i64, 
                                                                                frame_gap: i64, frame_dash2: i64, frame_gap2: i64,
                                                                                ftop: bool, fright: bool, fbottom: bool, fleft: bool,
                                                                                bullet: &str) -> Result<(), Box<dyn std::error::Error>> {
        match root_layout.type_name(){
            "NoneLayout" =>{
                if bullet != "" {
                    if has_frame{
                        let nlayout = elements::FramedElement::with_line_style_trbl(element,
                                                                                style::LineStyle::new()
                                                                                .with_thickness(frame_thickness)
                                                                                .with_color(frame_color)
                                                                                .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                    ftop, fright, fbottom, fleft);
                        self.doc.push(BulletPoint::new(nlayout).with_bullet(bullet));                                     
                    }else{
                        self.doc.push(BulletPoint::new(element).with_bullet(bullet)); 
                    }                                                                        
                }else{
                    if has_frame{
                        let nlayout = elements::FramedElement::with_line_style_trbl(element,
                                                                                style::LineStyle::new()
                                                                                .with_thickness(frame_thickness)
                                                                                .with_color(frame_color)
                                                                                .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                    ftop, fright, fbottom, fleft);
                        self.doc.push(nlayout);                                     
                    }else{
                        self.doc.push(element);
                    }                                    
                } 
            }
            "StaticList" =>{
                if bullet != "" {
                    if has_frame{
                        let nlayout = elements::FramedElement::with_line_style_trbl(element,
                                                                                style::LineStyle::new()
                                                                                .with_thickness(frame_thickness)
                                                                                .with_color(frame_color)
                                                                                .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                    ftop, fright, fbottom, fleft);
                        root_layout.push_static(BulletPoint::new(nlayout).with_bullet(bullet));                                     
                    }else{
                        root_layout.push_static(BulletPoint::new(element).with_bullet(bullet));
                    }                                      
                }else{
                    if has_frame{
                        let nlayout = elements::FramedElement::with_line_style_trbl(element,
                                                                                style::LineStyle::new()
                                                                                .with_thickness(frame_thickness)
                                                                                .with_color(frame_color)
                                                                                .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                    ftop, fright, fbottom, fleft);
                        root_layout.push_static(nlayout);                                      
                    }else{
                        root_layout.push_static(element);
                    }                                      
                }                                
            }
            "TableLayout" =>{
                //
            }  
            "VecLayout" =>{
                if bullet != "" {                                    
                    if has_frame{
                        let nlayout = elements::FramedElement::with_line_style_trbl(element,
                                                                                style::LineStyle::new()
                                                                                .with_thickness(frame_thickness)
                                                                                .with_color(frame_color)
                                                                                .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                    ftop, fright, fbottom, fleft);
                        root_layout.push_cell(Box::new(BulletPoint::new(nlayout).with_bullet(bullet)));                                      
                    }else{
                        root_layout.push_cell(Box::new(BulletPoint::new(element).with_bullet(bullet)));
                    }
                }else{
                    if has_frame{
                        let nlayout = elements::FramedElement::with_line_style_trbl(element,
                                                                                style::LineStyle::new()
                                                                                .with_thickness(frame_thickness)
                                                                                .with_color(frame_color)
                                                                                .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                    ftop, fright, fbottom, fleft);
                        root_layout.push_cell(Box::new(nlayout));                                        
                    }else{
                        root_layout.push_cell(Box::new(element));
                    }                                      
                }                                
            }
            _ => {
                if bullet != "" {
                    if has_frame{
                        let nlayout = elements::FramedElement::with_line_style_trbl(element,
                                                                                style::LineStyle::new()
                                                                                .with_thickness(frame_thickness)
                                                                                .with_color(frame_color)
                                                                                .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                    ftop, fright, fbottom, fleft);
                        root_layout.push(BulletPoint::new(nlayout).with_bullet(bullet));                                              
                    }else{
                        root_layout.push(BulletPoint::new(element).with_bullet(bullet));       
                    }                                                                       
                }else{
                    if has_frame{
                        let nlayout = elements::FramedElement::with_line_style_trbl(element,
                                                                                style::LineStyle::new()
                                                                                .with_thickness(frame_thickness)
                                                                                .with_color(frame_color)
                                                                                .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                    ftop, fright, fbottom, fleft);
                        root_layout.push(nlayout);                                        
                    }else{
                        root_layout.push(element);
                    }                                    
                }
            }
        }                        
        Ok(())
    }   
    
    fn push_element<T: RootLayout>(&mut self,  item_obj: &serde_json::Value, root_layout: &mut T) -> Result<(), Box<dyn std::error::Error>> {
        
            if let Some(etype) = item_obj.get("type").and_then(|v| v.as_str()){
                match etype{
                    "layout" => {
                        if let Some(orientation) = item_obj.get("orientation").and_then(|v| v.as_str()) {
                            if orientation == "vertical" {
                                
                                let mut mstyle = style::Style::new();
                                if let Some(style) = item_obj.get("style") {
                                    mstyle = self.get_style(&style);                                       
                                }
                                let mut _layout = LinearLayout::vertical();
                                
                                if let Some(orphan) = item_obj.get("orphan").and_then(|v| v.as_bool()) {
                                    _layout.set_orphan(orphan);
                                    if let Some(position) = item_obj.get("position").and_then(|v| v.as_array()) {
                                        let x = position[0].as_f64().unwrap_or(0.0) as f32;
                                        let y = position[1].as_f64().unwrap_or(0.0) as f32;
                                        _layout.set_orphan_position(x, y);
                                    }
                                }
                        
                                let mut layout = LLayout{root:_layout};
                                if let Some(elements) = item_obj.get("elements").and_then(|v| v.as_array()) {
                                    for element in elements {
                                        self.push_element(element, &mut layout)?;
                                    }                                    
                                }
                                
                                let mut has_frame = false;
                                let mut frame_thickness = 0.1;
                                let mut frame_color = style::Color::Rgb(0,0,0);
                                let (mut frame_dash, mut frame_gap, mut frame_dash2, mut frame_gap2) = (0, 0, 0, 0);
                                let (mut ftop, mut fright, mut fbottom, mut fleft) = (true, true, true, true);
                                if let Some(frame) = item_obj.get("frame").and_then(|v| v.as_object()) {
                                    if let Some(thickness)= frame.get("thickness").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)){
                                        frame_thickness = thickness;
                                    }
                                    if let Some(color) = frame.get("color"){
                                        frame_color = get_color(color);
                                    }                                                                      
                                    (frame_dash, frame_gap, frame_dash2, frame_gap2) = self.get_dashes(frame);
                                    (ftop, fright, fbottom, fleft) = self.get_sides(frame);
                                    has_frame = true
                                }
                                let nlayout = layout.root.styled(mstyle);
                                let paddings = self.get_paddings(item_obj);                        
                                let nlayout = PaddedElement::new(
                                    nlayout,
                                    Margins::trbl(paddings[0], paddings[1], paddings[2], paddings[3]),
                                ); 
                                match root_layout.type_name(){
                                    "NoneLayout" =>{ 
                                        if has_frame{                                            
                                            let nlayout = elements::FramedElement::with_line_style_trbl(nlayout,
                                                                                                   style::LineStyle::new()
                                                                                                   .with_thickness(frame_thickness)
                                                                                                   .with_color(frame_color)
                                                                                                   .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                                   .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                        ftop, fright, fbottom, fleft);                                            
                                            self.doc.push(nlayout);
                                        }else{
                                            self.doc.push(nlayout);
                                        }
                                    }
                                    "StaticList" =>{
                                        if has_frame{
                                            let nlayout = elements::FramedElement::with_line_style_trbl(nlayout,
                                                                                                   style::LineStyle::new()
                                                                                                   .with_thickness(frame_thickness)
                                                                                                   .with_color(frame_color)
                                                                                                   .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                                   .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                        ftop, fright, fbottom, fleft); 
                                            root_layout.push_static(nlayout);
                                        }else{
                                            root_layout.push_static(nlayout);
                                        }
                                        
                                    }
                                    "TableLayout" =>{
                                        // not used
                                    }   
                                    "VecLayout" =>{
                                        if has_frame{
                                            let nlayout = elements::FramedElement::with_line_style_trbl(nlayout,
                                                                                                   style::LineStyle::new()
                                                                                                   .with_thickness(frame_thickness)
                                                                                                   .with_color(frame_color)
                                                                                                   .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                                   .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                        ftop, fright, fbottom, fleft); 
                                            root_layout.push_cell(Box::new(nlayout));
                                        }else{
                                            root_layout.push_cell(Box::new(nlayout));
                                        }                                        
                                    }
                                    _ => {
                                        if has_frame{
                                            let nlayout = elements::FramedElement::with_line_style_trbl(nlayout,
                                                                                                   style::LineStyle::new()
                                                                                                   .with_thickness(frame_thickness)
                                                                                                   .with_color(frame_color)
                                                                                                   .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                                   .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                        ftop, fright, fbottom, fleft); 
                                            root_layout.push(nlayout);
                                        }else{
                                            root_layout.push(nlayout);
                                        }
                                        
                                    }
                                }                                                                                              
                            }else if orientation == "horizontal"{
                                if let Some(column_weights) = item_obj.get("column_weights").and_then(|v| v.as_array()) {
                                    if column_weights.len() > 0 {                                        
                                        let usize_values: Vec<usize> = column_weights
                                            .into_iter()
                                            .filter_map(|val| {
                                                if let serde_json::Value::Number(num) = val {
                                                    num.as_u64().and_then(|n| Some(n as usize))
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect();
                                        let mut mstyle = style::Style::new();
                                        if let Some(style) = item_obj.get("style") {
                                            mstyle = self.get_style(&style);                                       
                                        }
                                        let mut _horizontal_layout = TableLayout::new(usize_values);
                                        _horizontal_layout.set_cell_decorator(FrameCellDecorator::new(false,false,false));  
                                        
                                        let mut frame_thickness = 0.1;
                                        let mut frame_color = style::Color::Rgb(0,0,0);                                        
                                        if let Some(frame) = item_obj.get("frame").and_then(|v| v.as_object()) {
                                            if let Some(thickness)= frame.get("thickness").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)){
                                                frame_thickness = thickness;
                                            }
                                            if let Some(color) = frame.get("color"){
                                                frame_color = get_color(color);
                                            }
                                            let (frame_dash, frame_gap, frame_dash2, frame_gap2) = self.get_dashes(frame);
                                            _horizontal_layout.set_cell_decorator(FrameCellDecorator::with_line_style(false,true,false, 
                                                                                                                        style::LineStyle::new()
                                                                                                                        .with_thickness(frame_thickness)
                                                                                                                        .with_color(frame_color)
                                                                                                                        .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                                                        .with_gap(frame_gap).with_gap2(frame_gap2)));
                                        }                                        
                                        let mut horizontal_layout = TLayout{root:_horizontal_layout};
                                        let mut _vec_layout: Vec<Box<dyn Element>> = Vec::new();
                                        let mut vec_layout = VecLayout{root:_vec_layout};
                                        if let Some(elements) = item_obj.get("elements").and_then(|v| v.as_array()) {
                                            if elements.len() == column_weights.len(){                                                
                                                for element in elements {
                                                    self.push_element(element, &mut vec_layout)?;
                                                } 
                                                let _= horizontal_layout.push_row(vec_layout.root);
                                            }
                                        } 
                                        let nlayout = horizontal_layout.root.styled(mstyle);
                                        let paddings = self.get_paddings(item_obj);                        
                                        let nlayout = PaddedElement::new(
                                            nlayout,
                                            Margins::trbl(paddings[0], paddings[1], paddings[2], paddings[3]),
                                        );
                                        match root_layout.type_name(){
                                            "NoneLayout" =>{    
                                                self.doc.push(nlayout);
                                            }
                                            "StaticList" =>{
                                                root_layout.push_static(nlayout);
                                            }
                                            "TableLayout" =>{
                                                // not used
                                            }   
                                            "VecLayout" =>{
                                                root_layout.push_cell(Box::new(nlayout));
                                            }
                                            _ => {                                                
                                                root_layout.push(nlayout);
                                            }
                                        }                                                                                
                                    }                                    
                                }                                                                                                                                               
                            }
                        }
                    }
                    
                    "table_layout" => {
                        // inner: bool, outer: bool, cont: bool
                        let mut list_decorador: Vec<bool> = vec!(true, true, true);
                        if let Some(frame_decorator) = item_obj.get("frame_decorator").and_then(|v| v.as_array()) {
                            if frame_decorator.len() == 3 {
                                list_decorador.clear();
                                for it in frame_decorator{
                                    list_decorador.push(it.as_bool().expect("frame_decorator not bool"));
                                }
                            }
                        }
                        if let Some(column_weights) = item_obj.get("column_weights").and_then(|v| v.as_array()) {
                            if column_weights.len() > 0 {                                        
                                let usize_values: Vec<usize> = column_weights
                                    .into_iter()
                                    .filter_map(|val| {
                                        if let serde_json::Value::Number(num) = val {
                                            num.as_u64().and_then(|n| Some(n as usize))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                let mut mstyle = style::Style::new();
                                if let Some(style) = item_obj.get("style") {
                                    mstyle = self.get_style(&style);                                       
                                }    
                                let mut _table_layout = TableLayout::new(usize_values);
                                _table_layout.set_cell_decorator(FrameCellDecorator::new(list_decorador[0],list_decorador[1],list_decorador[2]));
                                
                                let mut frame_thickness = 0.1;
                                let mut frame_color = style::Color::Rgb(0,0,0);                                
                                if let Some(frame) = item_obj.get("frame").and_then(|v| v.as_object()) {
                                    if let Some(thickness)= frame.get("thickness").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)){
                                        frame_thickness = thickness;
                                    }
                                    if let Some(color) = frame.get("color"){
                                        frame_color = get_color(color);
                                    }                                    
                                    let (frame_dash, frame_gap, frame_dash2, frame_gap2) = self.get_dashes(frame);
                                    _table_layout.set_cell_decorator(FrameCellDecorator::with_line_style(list_decorador[0],list_decorador[1],list_decorador[2], 
                                                                                                                style::LineStyle::new()
                                                                                                                .with_thickness(frame_thickness)
                                                                                                                .with_color(frame_color)
                                                                                                                .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                                                .with_gap(frame_gap).with_gap2(frame_gap2)));
                                }  
                                
                                let mut table_layout = TLayout{root:_table_layout};
                                if let Some(rows) = item_obj.get("rows").and_then(|v| v.as_array()) {
                                    for row in rows{
                                        let mut _vec_layout: Vec<Box<dyn Element>> = Vec::new();
                                        let mut vec_layout = VecLayout{root:_vec_layout};
                                        if let Some(row_elementos) = row.as_array() {
                                            if row_elementos.len() == column_weights.len(){                                                
                                                for element in row_elementos {
                                                    self.push_element(element, &mut vec_layout)?;
                                                } 
                                                let _ = table_layout.push_row(vec_layout.root);
                                            }
                                        }        
                                    }
                                }else{
                                    // use sqlite in rows
                                    if let Some(tablename) = item_obj.get("rows").and_then(|v| v.as_str()) {                                     
                                        let conn = Connection::open(&self.db_path)?;
                                        let mut stmt = conn.prepare(format!("SELECT row FROM {} ORDER BY id ASC", tablename).as_str())?;
                                        let mut rows = stmt.query(params![])?;
                                        while let Some(row) = rows.next()? {            
                                            let data_string: String = row.get("row")?;
                                            let json_value: Value = serde_json::from_str(&data_string)?;
                                            
                                            let mut _vec_layout: Vec<Box<dyn Element>> = Vec::new();
                                            let mut vec_layout = VecLayout{root:_vec_layout};
                                            if let Some(row_elementos) = json_value.as_array() {
                                                if row_elementos.len() == column_weights.len(){                                                
                                                    for element in row_elementos {
                                                        self.push_element(element, &mut vec_layout)?;
                                                    } 
                                                    let _ = table_layout.push_row(vec_layout.root);
                                                }
                                            }                                                                                                                                       
                                        }                                        
                                    }
                                }
                                let nlayout = table_layout.root.styled(mstyle);
                                let paddings = self.get_paddings(item_obj);                        
                                let nlayout = PaddedElement::new(
                                    nlayout,
                                    Margins::trbl(paddings[0], paddings[1], paddings[2], paddings[3]),
                                );
                                match root_layout.type_name(){
                                    "NoneLayout" =>{    
                                        self.doc.push(nlayout);
                                    }
                                    "StaticList" =>{
                                        root_layout.push_static(nlayout);
                                    }
                                    "TableLayout" =>{
                                        // not used
                                    }   
                                    "VecLayout" =>{
                                        root_layout.push_cell(Box::new(nlayout));
                                    }
                                    _ => {                                                
                                        root_layout.push(nlayout);
                                    }
                                } 
                            }
                        }                                            
                    }
                    
                    "ordered_list" => {
                        let mut _order_list = OrderedList::new();
                        if let Some(start) = item_obj.get("start").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as usize)){
                            if start > 1 {
                                _order_list = OrderedList::with_start(start);
                            }
                        }
                        let mut order_list = OLayout{root: _order_list};
                        if let Some(elements) = item_obj.get("elements").and_then(|v| v.as_array()) {
                            for element in elements {
                                self.push_element(element, &mut order_list)?;
                            }                                    
                        }    
                        let mut has_frame = false;
                        let mut frame_thickness = 0.1;
                        let mut frame_color = style::Color::Rgb(0,0,0);
                        let (mut frame_dash, mut frame_gap, mut frame_dash2, mut frame_gap2) = (0, 0, 0, 0);
                        let (mut ftop, mut fright, mut fbottom, mut fleft) = (true, true, true, true);
                        if let Some(frame) = item_obj.get("frame").and_then(|v| v.as_object()) {
                            if let Some(thickness)= frame.get("thickness").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)){
                                frame_thickness = thickness;
                            }
                            if let Some(color) = frame.get("color"){
                                frame_color = get_color(color);
                            }
                            (frame_dash, frame_gap, frame_dash2, frame_gap2) = self.get_dashes(frame);
                            (ftop, fright, fbottom, fleft) = self.get_sides(frame);
                            has_frame = true
                        }
                        let mut mstyle = style::Style::new();
                        if let Some(style) = item_obj.get("style") {
                            mstyle = self.get_style(&style);                                       
                        }
                        let nlayout = order_list.root.styled(mstyle);
                        let paddings = self.get_paddings(item_obj);                        
                        let nlayout = PaddedElement::new(
                            nlayout,
                            Margins::trbl(paddings[0], paddings[1], paddings[2], paddings[3]),
                        );
                        match root_layout.type_name(){
                            "NoneLayout" =>{         
                                if has_frame{
                                    let nlayout = elements::FramedElement::with_line_style_trbl(nlayout,
                                                                                            style::LineStyle::new()
                                                                                            .with_thickness(frame_thickness)
                                                                                            .with_color(frame_color)
                                                                                            .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                            .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                ftop, fright, fbottom, fleft); 
                                    self.doc.push(nlayout);
                                }else{
                                    self.doc.push(nlayout);
                                }                                 
                            }
                            "StaticList" =>{
                                if has_frame{
                                    let nlayout = elements::FramedElement::with_line_style_trbl(nlayout,
                                                                                            style::LineStyle::new()
                                                                                            .with_thickness(frame_thickness)
                                                                                            .with_color(frame_color)
                                                                                            .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                            .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                ftop, fright, fbottom, fleft); 
                                    root_layout.push_static(nlayout);
                                }else{
                                    root_layout.push_static(nlayout);
                                }                                 
                            }
                            "TableLayout" =>{
                                //
                            }  
                            "VecLayout" =>{
                                if has_frame{
                                    let nlayout = elements::FramedElement::with_line_style_trbl(nlayout,
                                                                                            style::LineStyle::new()
                                                                                            .with_thickness(frame_thickness)
                                                                                            .with_color(frame_color)
                                                                                            .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                            .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                ftop, fright, fbottom, fleft); 
                                    root_layout.push_cell(Box::new(nlayout));
                                }else{
                                    root_layout.push_cell(Box::new(nlayout));
                                }                                 
                            }
                            _ => {
                                if has_frame{
                                    let nlayout = elements::FramedElement::with_line_style_trbl(nlayout,
                                                                                            style::LineStyle::new()
                                                                                            .with_thickness(frame_thickness)
                                                                                            .with_color(frame_color)
                                                                                            .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                            .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                ftop, fright, fbottom, fleft); 
                                    root_layout.push(nlayout);
                                }else{
                                    root_layout.push(nlayout);
                                }                                
                            }
                        }
                    }
                    
                    "unordered_list" => {
                        let mut _unorder_list = UnorderedList::new();
                        if let Some(bullet) = item_obj.get("bullet").and_then(|v| v.as_str()) {
                            if bullet != ""{
                                _unorder_list = UnorderedList::with_bullet(bullet);
                            }
                        }
                        let mut unorder_list = ULayout{root: _unorder_list};
                        if let Some(elements) = item_obj.get("elements").and_then(|v| v.as_array()) {
                            for element in elements {
                                self.push_element(element, &mut unorder_list)?;
                            }                                    
                        } 
                        let mut has_frame = false;
                        let mut frame_thickness = 0.1;
                        let mut frame_color = style::Color::Rgb(0,0,0);
                        let (mut frame_dash, mut frame_gap, mut frame_dash2, mut frame_gap2) = (0, 0, 0, 0);
                        let (mut ftop, mut fright, mut fbottom, mut fleft) = (true, true, true, true);
                        if let Some(frame) = item_obj.get("frame").and_then(|v| v.as_object()) {
                            if let Some(thickness)= frame.get("thickness").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)){
                                frame_thickness = thickness;
                            }
                            if let Some(color) = frame.get("color"){
                                frame_color = get_color(color);
                            }
                            (frame_dash, frame_gap, frame_dash2, frame_gap2) = self.get_dashes(frame);
                            (ftop, fright, fbottom, fleft) = self.get_sides(frame);
                            has_frame = true
                        }                        
                        
                        let mut mstyle = style::Style::new();
                        if let Some(style) = item_obj.get("style") {
                            mstyle = self.get_style(&style);                                       
                        }
                        let nlayout = unorder_list.root.styled(mstyle);
                        let paddings = self.get_paddings(item_obj);                        
                        let nlayout = PaddedElement::new(
                            nlayout,
                            Margins::trbl(paddings[0], paddings[1], paddings[2], paddings[3]),
                        );
                        match root_layout.type_name(){
                            "NoneLayout" =>{
                                if has_frame{
                                    let nlayout = elements::FramedElement::with_line_style_trbl(nlayout,
                                                                                            style::LineStyle::new()
                                                                                            .with_thickness(frame_thickness)
                                                                                            .with_color(frame_color)
                                                                                            .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                            .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                ftop, fright, fbottom, fleft); 
                                    self.doc.push(nlayout);
                                }else{
                                    self.doc.push(nlayout);
                                }
                            }
                            "StaticList" =>{
                                if has_frame{
                                    let nlayout = elements::FramedElement::with_line_style_trbl(nlayout,
                                                                                            style::LineStyle::new()
                                                                                            .with_thickness(frame_thickness)
                                                                                            .with_color(frame_color)
                                                                                            .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                            .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                ftop, fright, fbottom, fleft); 
                                    root_layout.push_static(nlayout);
                                }else{
                                    root_layout.push_static(nlayout);
                                }
                            }
                            "TableLayout" =>{
                                //
                            }  
                            "VecLayout" =>{
                                if has_frame{
                                    let nlayout = elements::FramedElement::with_line_style_trbl(nlayout,
                                                                                            style::LineStyle::new()
                                                                                            .with_thickness(frame_thickness)
                                                                                            .with_color(frame_color)
                                                                                            .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                            .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                ftop, fright, fbottom, fleft); 
                                    root_layout.push_cell(Box::new(nlayout));
                                }else{
                                    root_layout.push_cell(Box::new(nlayout));
                                }
                            }
                            _ => {
                                if has_frame{
                                    let nlayout = elements::FramedElement::with_line_style_trbl(nlayout,
                                                                                            style::LineStyle::new()
                                                                                            .with_thickness(frame_thickness)
                                                                                            .with_color(frame_color)
                                                                                            .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                            .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                ftop, fright, fbottom, fleft); 
                                    root_layout.push(nlayout);
                                }else{
                                    root_layout.push(nlayout);
                                }
                            }
                        }
                    }
                    
                    "break" => {
                        if let Some(value) = item_obj.get("value").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)) {
                            let element = Break::new(value);
                            //allow line break in negative
                            // if value > 0.0 {
                                match root_layout.type_name(){
                                    "NoneLayout" =>{                                                
                                        self.doc.push(element);
                                    }
                                    "StaticList" =>{
                                        root_layout.push_static(element);
                                    }
                                    "TableLayout" =>{
                                        //
                                    }  
                                    "VecLayout" =>{
                                        root_layout.push_cell(Box::new(element));
                                    }
                                    _ => {
                                        root_layout.push(element);
                                    }
                                }
                            // }
                        }
                    }
                    
                    "page_break" => {                       
                        let element = PageBreak::new();
                        self.doc.push(element);                                                  
                    }
                    
                    "image" => {   
                        let mut path =  "".to_string();
                        let mut base64 = "".to_string();
                        if let Some(_path) = item_obj.get("path").and_then(|v| v.as_str()) {
                            path = _path.to_string();
                        }   
                        if let Some(_base64) = item_obj.get("base64").and_then(|v| v.as_str()) {
                            base64 = _base64.to_string();
                        }
                        if path != "".to_string() || base64 != "".to_string() {
                            let mut image = if path != "".to_string() {
                                rckive_genpdf::elements::Image::from_path(path).expect("Unable to load image")
                            }else{
                                rckive_genpdf::elements::Image::from_base64(&base64).expect("Unable to load image")
                            };
                            
                            if let Some(str_alignment) = item_obj.get("alignment").and_then(|v| v.as_str()) {
                                if let Some(alignment) = self.alignment_map.get(str_alignment) { 
                                    image.set_alignment(*alignment);
                                }
                            }
                            if let Some(position) = item_obj.get("position").and_then(|v| v.as_array()) {
                                    if position.len() >= 2 {
                                        let posx = position[0].as_f64().unwrap_or(0.0) as f32;
                                        let posy = position[1].as_f64().unwrap_or(0.0) as f32;
                                        image.set_position(rckive_genpdf::Position::new(posx, posy));
                                    }
                            }
                            if let Some(rotation) = item_obj.get("rotation").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)) {
                                image.set_clockwise_rotation(rotation);
                            }
                            if let Some(dpi) = item_obj.get("dpi").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)) {
                                image.set_dpi(dpi);
                            }
                            if let Some(scale) = item_obj.get("scale").and_then(|v| v.as_array()) {
                                if scale.len()>=2{
                                    let s1 = scale[0].as_f64().unwrap_or(0.0) as f32;
                                    let s2 = scale[1].as_f64().unwrap_or(0.0) as f32;
                                    image.set_scale(rckive_genpdf::Scale::new(s1,s2));
                                }
                            }else{
                                if let Some(scale) = item_obj.get("scale").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)) {                            
                                    image.set_scale(rckive_genpdf::Scale::new(scale,scale));
                                }
                            }
                            let mut has_frame = false;
                            let mut frame_thickness = 0.1;
                            let mut frame_color = style::Color::Rgb(0,0,0);
                            let (mut frame_dash, mut frame_gap, mut frame_dash2, mut frame_gap2) = (0, 0, 0, 0);
                            let (mut ftop, mut fright, mut fbottom, mut fleft) = (true, true, true, true);
                            if let Some(frame) = item_obj.get("frame").and_then(|v| v.as_object()) {
                                if let Some(thickness)= frame.get("thickness").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)){
                                    frame_thickness = thickness;
                                }
                                if let Some(color) = frame.get("color"){
                                    frame_color = get_color(color);
                                }
                                (frame_dash, frame_gap, frame_dash2, frame_gap2) = self.get_dashes(frame);
                                (ftop, fright, fbottom, fleft) = self.get_sides(frame);
                                has_frame = true
                            }
                            let paddings = self.get_paddings(item_obj);                        
                            let image = PaddedElement::new(
                                image,
                                Margins::trbl(paddings[0], paddings[1], paddings[2], paddings[3]),
                            );
                            match root_layout.type_name(){
                                    "NoneLayout" =>{       
                                        if has_frame{
                                            let nlayout = elements::FramedElement::with_line_style_trbl(image,
                                                                                            style::LineStyle::new()
                                                                                            .with_thickness(frame_thickness)
                                                                                            .with_color(frame_color)
                                                                                            .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                            .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                ftop, fright, fbottom, fleft); 
                                            self.doc.push(nlayout);
                                        }else{
                                            self.doc.push(image);
                                        }                                        
                                    }
                                    "StaticList" =>{
                                        if has_frame{
                                            let nlayout = elements::FramedElement::with_line_style_trbl(image,
                                                                                            style::LineStyle::new()
                                                                                            .with_thickness(frame_thickness)
                                                                                            .with_color(frame_color)
                                                                                            .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                            .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                ftop, fright, fbottom, fleft); 
                                            root_layout.push_static(nlayout);
                                        }else{
                                            root_layout.push_static(image);
                                        }                                         
                                    }
                                    "TableLayout" =>{
                                        //
                                    }  
                                    "VecLayout" =>{
                                        if has_frame{
                                            let nlayout = elements::FramedElement::with_line_style_trbl(image,
                                                                                            style::LineStyle::new()
                                                                                            .with_thickness(frame_thickness)
                                                                                            .with_color(frame_color)
                                                                                            .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                            .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                ftop, fright, fbottom, fleft); 
                                            root_layout.push_cell(Box::new(nlayout));
                                        }else{
                                            root_layout.push_cell(Box::new(image));
                                        }                                        
                                    }
                                    _ => {
                                        if has_frame{
                                            let nlayout = elements::FramedElement::with_line_style_trbl(image,
                                                                                            style::LineStyle::new()
                                                                                            .with_thickness(frame_thickness)
                                                                                            .with_color(frame_color)
                                                                                            .with_dash(frame_dash).with_dash2(frame_dash2)
                                                                                            .with_gap(frame_gap).with_gap2(frame_gap2),
                                                                                                ftop, fright, fbottom, fleft); 
                                            root_layout.push(nlayout);
                                        }else{
                                            root_layout.push(image);
                                        }                                        
                                    }
                            }                                                        
                        }
                    }
                    "paragraph" => {                                          
                        // only Paragraph implemented
                        let mut element = Paragraph::default();
                                             
                        //alignment
                        if let Some(str_alignment) = item_obj.get("alignment").and_then(|v| v.as_str()) {
                            if let Some(alignment) = self.alignment_map.get(str_alignment) { 
                                element.set_alignment(*alignment);
                            }
                        }
                        
                        let mstyle = style::Style::new();
                        if let Some(value) = item_obj.get("value").and_then(|v| v.as_array()) {
                            for val_style in value {      
                                let mstyle = self.get_style(&val_style);                                
                                if let Some(text) = val_style.get("text").and_then(|v| v.as_str()) {
                                    element.push_styled(text, mstyle);         
                                }                                
                            }
                        }else{
                            if let Some(value) = item_obj.get("value").and_then(|v| v.as_str()) {
                                element.push_styled(value, mstyle);  
                            }
                        }                               
                        // top, right, bottom, left
                        let paddings = self.get_paddings(item_obj);                        
                        let element = PaddedElement::new(
                                    element,
                                    Margins::trbl(paddings[0], paddings[1], paddings[2], paddings[3]),
                                );  
                        let mut has_frame = false;
                        let mut frame_thickness = 0.1;
                        let mut frame_color = style::Color::Rgb(0,0,0);
                        let (mut frame_dash, mut frame_gap, mut frame_dash2, mut frame_gap2) = (0, 0, 0, 0);
                        let (mut ftop, mut fright, mut fbottom, mut fleft) = (true, true, true, true);
                        if let Some(frame) = item_obj.get("frame").and_then(|v| v.as_object()) {
                            if let Some(thickness)= frame.get("thickness").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)){
                                frame_thickness = thickness;
                            }
                            if let Some(color) = frame.get("color"){
                                frame_color = get_color(color);
                            }
                            (frame_dash, frame_gap, frame_dash2, frame_gap2) = self.get_dashes(frame);
                            (ftop, fright, fbottom, fleft) = self.get_sides(frame);
                            has_frame = true
                        }                        
                        let mut bullet = "";
                        if let Some(bullet_point) = item_obj.get("bullet").and_then(|v| v.as_str()) {
                            bullet = bullet_point
                        }
                        self.match_text_paragraph(element, root_layout, has_frame,
                                                  frame_thickness, frame_color, frame_dash, frame_gap, 
                                                  frame_dash2, frame_gap2, ftop, fright, fbottom, fleft,
                                                  bullet)?;
                    }
                        
                    "text" => {
                        let mut element = Text::default();
                        if let Some(value) = item_obj.get("value").and_then(|v| v.as_str()) {
                            if let Some(style) = item_obj.get("style") {
                                let mstyle = self.get_style(&style);   
                                element = Text::new(style::StyledString::new(value, mstyle));
                            }else{                                
                                element = Text::new(value);
                            }
                        }
                        if let Some(orphan) = item_obj.get("orphan").and_then(|v| v.as_bool()) {
                            element.set_orphan(orphan);
                            if let Some(position) = item_obj.get("position").and_then(|v| v.as_array()) {
                                let x = position[0].as_f64().unwrap_or(0.0) as f32;
                                let y = position[1].as_f64().unwrap_or(0.0) as f32;
                                element.set_orphan_position(x, y);
                            }
                        }
                        // // top, right, bottom, left
                        let paddings = self.get_paddings(item_obj);                        
                        let element = PaddedElement::new(
                                    element,
                                    Margins::trbl(paddings[0], paddings[1], paddings[2], paddings[3]),
                                );
                        let mut has_frame = false;
                        let mut frame_thickness = 0.1;
                        let mut frame_color = style::Color::Rgb(0,0,0);                        
                        let (mut frame_dash, mut frame_gap, mut frame_dash2, mut frame_gap2) = (0, 0, 0, 0);
                        let (mut ftop, mut fright, mut fbottom, mut fleft) = (true, true, true, true);
                        if let Some(frame) = item_obj.get("frame").and_then(|v| v.as_object()) {
                            if let Some(thickness)= frame.get("thickness").and_then(|v| Some(v.as_f64().unwrap_or(0.0) as f32)){
                                frame_thickness = thickness;
                            }
                            if let Some(color) = frame.get("color"){
                                frame_color = get_color(color);
                            }
                            (frame_dash, frame_gap, frame_dash2, frame_gap2) = self.get_dashes(frame);
                            (ftop, fright, fbottom, fleft) = self.get_sides(frame);
                            has_frame = true
                        }                      
                        let mut bullet = "";
                        if let Some(bullet_point) = item_obj.get("bullet").and_then(|v| v.as_str()) {
                            bullet = bullet_point
                        }
                        self.match_text_paragraph(element, root_layout, has_frame,
                                                  frame_thickness, frame_color, frame_dash, frame_gap, 
                                                  frame_dash2, frame_gap2, ftop, fright, fbottom, fleft,
                                                  bullet)?;
                                              
                    },
                    _ => println!("The JSON does not have the expected type."),                
                }
            }                
         Ok(())
    }    
}
/// render file pdf from file json
pub fn render_json_file(json_path: impl AsRef<path::Path>, path: impl AsRef<path::Path>) -> Result<(), Box<dyn std::error::Error>> {
        let mut config_default = json!({        
                "fonts":[
                    // {"font_family_name":"LiberationSans",  "dir":"/usr/share/fonts/truetype/liberation"}
                ],
                "title": "Report GenPdfJson",
                "default_font":{"font_family_name":"LiberationSans",  "dir":"/usr/share/fonts/truetype/liberation"},
                "margins":10,
                "skip_warning_overflowed": true               
        });
        let json_string = fs::read_to_string(json_path)?;
        let json_value: Value = serde_json::from_str(&json_string)?;
        if let Some(config) = json_value.get("config") {
            config_default = config.clone();        
        }
        
        println!("init ...");
        let mut genpdf = GenpdfJson::new(&config_default, "");  
        genpdf.extra_push(&config_default)?;
        genpdf.render_json_file(&json_value, path)?;
        println!("generated successfully");
        Ok(())
    }
/// render pdf in memory and return string base64 from file json
pub fn render_json_base64(json_string: &String) -> Result<String, Box<dyn std::error::Error>> {
        let mut config_default = json!({        
                "fonts":[
                    // {"font_family_name":"LiberationSans",  "dir":"/usr/share/fonts/truetype/liberation"}
                ],
                "title": "Report GenPdfJson",
                "default_font":{"font_family_name":"LiberationSans",  "dir":"/usr/share/fonts/truetype/liberation"},
                "margins":10,
                "skip_warning_overflowed": true
        });      
        let json_value: Value = serde_json::from_str(&json_string)?;
        if let Some(config) = json_value.get("config") {
            config_default = config.clone();        
        }        
        let mut genpdf = GenpdfJson::new(&config_default, "");
        genpdf.extra_push(&config_default)?;
        let result = genpdf.render_json_base64(&json_value)?;        
        Ok(result)
    }
/// render file pdf from sqlite .db
pub fn render_file_from_sqlite(db_path: impl AsRef<path::Path>, path: impl AsRef<path::Path>) -> Result<(), Box<dyn std::error::Error>> {
        let mut config_default = json!({        
                "fonts":[
                    // {"font_family_name":"LiberationSans",  "dir":"/usr/share/fonts/truetype/liberation"}
                ],
                "title": "Report GenPdfJson",
                "default_font":{"font_family_name":"LiberationSans",  "dir":"/usr/share/fonts/truetype/liberation"},
                "margins":10,
                "skip_warning_overflowed": true
        });
        
        let conn = Connection::open(&db_path)?;
        let mut stmt = conn.prepare("SELECT data FROM config LIMIT 1")?;
        
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let data_string: String = row.get("data")?;
            config_default = serde_json::from_str(&data_string)?;
        }    
                
        println!("init ...");
        let mut genpdf = GenpdfJson::new(&config_default, &db_path); 
        genpdf.extra_push(&config_default)?;
        genpdf.render_file_from_sqlite(&db_path, path)?;
        println!("generated successfully");
        Ok(())
}
/// render pdf in memory and return string base64 from sqlite .db
pub fn render_base64_from_sqlite(db_path: impl AsRef<path::Path>) -> Result<String, Box<dyn std::error::Error>> {
    let mut config_default = json!({        
            "fonts":[
                // {"font_family_name":"LiberationSans",  "dir":"/usr/share/fonts/truetype/liberation"}
            ],
            "title": "Report GenPdfJson",
            "default_font":{"font_family_name":"LiberationSans",  "dir":"/usr/share/fonts/truetype/liberation"},
            "margins":10,
            "skip_warning_overflowed": true
    });      
    let conn = Connection::open(&db_path)?;
        let mut stmt = conn.prepare("SELECT data FROM config LIMIT 1")?;
        
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let data_string: String = row.get("data")?;
            config_default = serde_json::from_str(&data_string)?;
        }          
    let mut genpdf = GenpdfJson::new(&config_default, &db_path);
    genpdf.extra_push(&config_default)?;
    let result = genpdf.render_base64_from_sqlite(&db_path)?;        
    Ok(result)
}    
/// Read the version from build
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
