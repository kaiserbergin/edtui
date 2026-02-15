# Line Management on `main` Branch

This document describes how **actual text lines**, **rendered lines**, **spans**, **internal spans**, and **selections** work together in the edtui codebase on the **main** branch.

---

## 1. Data Model Overview

```mermaid
erDiagram
    Lines ||--o{ Row : "has rows"
    Lines {
        Jagged_char buffer
    }
    Row {
        Vec_char content
    }
    Selection ||--|| Index2 : "start"
    Selection ||--|| Index2 : "end"
    Selection {
        bool line_mode
        Index2 anchor
    }
    Index2 {
        int row
        int col
    }
    ViewState ||--|| Offset : "viewport"
    ViewState {
        int num_rows
        bool wrap
        int tab_width
        Rect screen_area
    }
    Offset {
        int x
        int y
    }
```

- **`Lines`** (`jagged::Jagged<char>`): The buffer. Each **row** is one logical line (`Vec<char>`). Indexed by `RowIndex`.
- **`Index2`**: `(row, col)` in **logical** coordinates (row = logical line index, col = character index).
- **`Selection`**: `start`, `end` (`Index2`), optional `line_mode` and `anchor`. All coordinates are in **logical** space (lines and columns).
- **`ViewState`**: Viewport offset `(x, y)` (column/row scroll), `num_rows` (number of logical lines visible), wrap/tab/line-number settings.

---

## 2. Coordinate Spaces

```mermaid
flowchart LR
    subgraph Logical["Logical (buffer) space"]
        L[Lines]
        R[Row index]
        C[Col index]
    end
    
    subgraph Rendered["Rendered (screen) space"]
        SR[Screen row]
        SC[Screen col]
    end
    
    Logical -->|"viewport offset, wrap / no wrap"| Rendered
```

| Space        | Row meaning              | Col meaning        | Used by                    |
|-------------|---------------------------|--------------------|----------------------------|
| **Logical** | Index into `Lines` (line) | Character index    | `Selection`, `Index2`, cursor |
| **Screen**  | Y in terminal            | X in terminal      | Cursor draw, mouse → logical |

- **No wrap:** 1 logical line → 1 screen row; horizontal scroll = `viewport.x`.
- **Wrap:** 1 logical line → 1+ screen rows (via `LineWrapper`); vertical scroll = `viewport.y` over **logical** lines; `find_position_in_wrapped_spans` maps logical column to (wrapped row, screen col).

---

## 3. Span Types

```mermaid
flowchart TB
    subgraph Ratatui["Ratatui (borrowed)"]
        Span["Span"]
    end
    
    subgraph Internal["Internal (owned)"]
        InternalSpan["InternalSpan"]
    end
    
    Span -->|"From trait (owned copy)"| InternalSpan
    InternalSpan -->|"Into Span"| Span
    
    Span -->|"content: &str, style"| S1[" "]
    InternalSpan -->|"content: String, style"| S2[" "]
```

- **`Span<'a>`** (ratatui): Borrowed content, used for rendering and in `LineWrapper::wrap_spans`.
- **`InternalSpan`**: Owned `String` + `Style`. Used for syntax highlighting and selection splitting so we can mutate and pass without lifetimes. Converts to/from `Span` for the rest of the pipeline.

---

## 4. Render Pipeline (main branch)

```mermaid
flowchart TB
    subgraph Input["Input"]
        Lines["Lines (buffer)"]
        Selections["Selections (e.g. selection + search)"]
        ViewState["ViewState (viewport, wrap, tab_width)"]
    end
    
    subgraph PerLogicalLine["Per logical line (from viewport offset)"]
        A["line = lines.iter_row().skip(offset_y)"]
        B["col_skips = offset_x"]
        C["generate_spans(line, selections, row_index, col_skips, ...)"]
        D{"wrap?"}
        E["LineWrapper::wrap_spans(spans, width, tab_width)"]
        F["RenderLine::Single(spans)"]
        G["RenderLine::Wrapped(wrapped_spans)"]
    end
    
    subgraph Output["Output"]
        H["RenderLine::render(area, buf, tab_width)"]
        I["data_coordinate_to_screen_coordinate for cursor"]
    end
    
    Lines --> A
    ViewState --> B
    A --> C
    Selections --> C
    C --> D
    D -->|yes| E
    D -->|no| F
    E --> G
    G --> H
    F --> H
    G --> I
    F --> I
```

**Order on main: selection → then wrap.**

1. For each visible logical line, **`generate_spans`** builds `Vec<Span>` with **selection (and syntax) already applied** (`line_into_spans_with_selections` or highlighted variant).
2. If **wrap** is on, **`LineWrapper::wrap_spans`** wraps these already-styled spans. Wrap boundaries can fall at span boundaries (e.g. selection/syntax boundaries).
3. Result is **`RenderLine::Single(spans)`** or **`RenderLine::Wrapped(Vec<Vec<Span>>)`**, then rendered and used for cursor position.

---

## 5. Selection Application (main)

```mermaid
sequenceDiagram
    participant V as view.rs
    participant Internal as internal.rs
    participant Sel as Selection
    
    V->>Internal: line_into_spans_with_selections(line, selections, row_index, col_skips, ...)
    loop For each char in line (skip col_skips)
        Internal->>Sel: selection.contains(&position)
        Sel-->>Internal: bool
        Note over Internal: On boundary change: push span (base or highlight style)
    end
    Internal-->>V: Vec Span (selection already split into spans)
```

- Selection is **per character**: for each `Index2(row_index, col)` we check `selection.contains(&pos)` and switch between base and highlight style. So we get **multiple spans per line** (normal vs selected).
- **Syntax highlighting (main):** `line_into_highlighted_spans_with_selections` first gets highlighted `InternalSpan`s, then **splits at selection** via `InternalSpan::split_at_selection` (which uses `split_spans` at `(start_col, end_col)`), then `crop_spans(col_skips)` for horizontal scroll. Result converted to `Span`s.

---

## 6. InternalSpan and Splitting

```mermaid
flowchart LR
    subgraph Split["split_spans(spans, split_start, split_end, style)"]
        A[Span A] --> B{overlap?}
        B -->|before| C[unchanged]
        B -->|after| D[unchanged]
        B -->|front/back/middle/none| E[split and apply style]
    end
    
    subgraph Crop["crop_spans(spans, crop_at)"]
        F[Drop spans before crop_at]
        G[Split first visible span at crop_at]
    end
```

- **`split_spans`**: Given a single logical row’s spans and a column range `(split_start, split_end)`, produces new spans with the given `style` in that range (for selection/syntax). Handles overlap cases (span fully inside, overlaps start/end, or spans multiple).
- **`crop_spans`**: Horizontal scroll: drop or trim spans so that the first visible character is at `crop_at` (character index).

---

## 7. Cursor and Viewport

```mermaid
flowchart TB
    Cursor["Cursor (row, col) in logical space"]
    Viewport["ViewState.viewport (x, y)"]
    
    Cursor --> Update["update_viewport_horizontal / _vertical / _vertical_wrap"]
    Update --> Viewport
    
    Viewport --> Iter["Iterate lines from offset_y, col_skips = offset_x"]
    Iter --> RenderLine["Build RenderLine per logical line"]
    RenderLine --> ScreenPos["data_coordinate_to_screen_coordinate(cursor.col - offset_x, ...)"]
    ScreenPos --> Draw["Draw cursor at Position"]
```

- **Scroll:** Viewport is updated so the cursor stays visible (scroll left/right without wrap, or scroll up/down by logical line with wrap).
- **Cursor draw:** For the logical line that contains the cursor, we call `render_line.data_coordinate_to_screen_coordinate(cursor.col - offset_x, area, tab_width)`. For wrapped mode this uses **`find_position_in_wrapped_spans`** (logical column → wrapped row + display width col).

---

## 8. Summary Diagram (main)

```mermaid
flowchart TB
    L["Lines = Jagged(char)"]
    S["Selection: Index2 start/end"]
    
    L --> LL[Logical line]
    S --> LL
    
    LL --> Gen["generate_spans (selection + syntax in)"]
    Gen --> Spans["Vec Span"]
    Spans --> Wrap["wrap_spans (if wrap)"]
    Wrap --> RL["RenderLine"]
    RL --> Screen["Screen (Buffer)"]
    
    Cursor["Cursor Index2"] --> RL
    RL --> CursorPos["Screen Position"]
```

**Takeaway (main):** Everything is driven by **logical lines** and **logical (row, col)**. Spans are produced **with selection (and syntax) applied first**; then wrapping is applied to those spans. Selection and span boundaries can therefore affect where wrapping occurs.
