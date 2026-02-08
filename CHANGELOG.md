# v0.1.14 (2026-02-05)
- Add page frame, footer frame and header frame, with line style.
  To activate, add to the config:
  "header_frame": line_style
  "footer_frame": line_style
  "page_frame": line_style
- Also, page frame width offset; negative values ​​reduce

# v0.1.13 (2025-12-18)
- allow assigning the `line_spacing` of each style to the element. This would allow for different elements with their respective line spacing.

# v0.1.12 (2025-12-02)
- Adding background to the frame, thanks to the advanced graphical options of printpdf-rs:
- Updating `GenpdfJson` to enable this new improvement.
- Replaced `with_line_style_trbl` `with with_line_style_trbl_and_background`.
- Capture JSON values `​​background` and `background_color`.
- Simplify some code for the match layout, in FrameElement and in background color (default white).

# v0.1.10 (2025-11) 
- add experimental footer
- Allow the use of 3 paragraphs in the footer and header
- update the elements of the genpdf changes
- Allow orphaned LinearLayout in the chosen position, avoiding warning.
- add extra_push in GenpdfJson: to add layouts on a different layer without margins
