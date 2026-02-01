use crate::node::{Display, Node, NodeId, NodeKind, Options, Wrap};
use crate::validators::Validator;
use crossterm::style::Color;

/// Type of date/time segment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    Year,   // YYYY
    Month,  // MM
    Day,    // DD
    Hour,   // HH
    Minute, // mm
    Second, // ss
}

impl SegmentType {
    /// Get min value for this segment type
    fn min_value(&self) -> u32 {
        match self {
            SegmentType::Year => 1900,
            SegmentType::Month => 1,
            SegmentType::Day => 1,
            SegmentType::Hour => 0,
            SegmentType::Minute => 0,
            SegmentType::Second => 0,
        }
    }

    /// Get max value for this segment type
    fn max_value(&self) -> u32 {
        match self {
            SegmentType::Year => 2100,
            SegmentType::Month => 12,
            SegmentType::Day => 31, // TODO: could be smarter based on month/year
            SegmentType::Hour => 23,
            SegmentType::Minute => 59,
            SegmentType::Second => 59,
        }
    }

    /// Get expected length (number of digits)
    fn length(&self) -> usize {
        match self {
            SegmentType::Year => 4,
            _ => 2,
        }
    }

    /// Parse from format string token
    fn from_token(token: &str) -> Option<Self> {
        match token {
            "YYYY" => Some(SegmentType::Year),
            "MM" => Some(SegmentType::Month),
            "DD" => Some(SegmentType::Day),
            "HH" => Some(SegmentType::Hour),
            "mm" => Some(SegmentType::Minute),
            "ss" => Some(SegmentType::Second),
            _ => None,
        }
    }
}

/// A single segment in the date input (e.g., day, month, year)
#[derive(Debug, Clone)]
pub struct DateSegment {
    pub segment_type: SegmentType,
    pub value: String, // Current value as string (can be partial, e.g., "2" for day)
}

impl DateSegment {
    fn new(segment_type: SegmentType) -> Self {
        Self {
            segment_type,
            value: String::new(),
        }
    }

    /// Get the numeric value (0 if empty or invalid)
    fn numeric_value(&self) -> u32 {
        self.value.parse().unwrap_or(0)
    }

    /// Increment value (with wraparound)
    fn increment(&mut self) {
        let current = self.numeric_value();
        let max = self.segment_type.max_value();
        let min = self.segment_type.min_value();

        let next = if current >= max || current < min {
            min
        } else {
            current + 1
        };

        self.value = format!("{:0width$}", next, width = self.segment_type.length());
    }

    /// Decrement value (with wraparound)
    fn decrement(&mut self) {
        let current = self.numeric_value();
        let max = self.segment_type.max_value();
        let min = self.segment_type.min_value();

        let prev = if current <= min || current == 0 {
            max
        } else {
            current - 1
        };

        self.value = format!("{:0width$}", prev, width = self.segment_type.length());
    }

    /// Insert a digit character at the end
    fn insert_digit(&mut self, digit: char) -> bool {
        if !digit.is_ascii_digit() {
            return false;
        }

        let max_len = self.segment_type.length();
        if self.value.len() >= max_len {
            // Replace all with new digit if full
            self.value = digit.to_string();
            return true;
        }

        self.value.push(digit);

        // Validate it doesn't exceed max
        if let Ok(val) = self.value.parse::<u32>() {
            if val > self.segment_type.max_value() {
                // If it exceeds, just set to the digit
                self.value = digit.to_string();
            }
        }

        true
    }

    /// Delete last digit
    fn delete_digit(&mut self) -> bool {
        if self.value.is_empty() {
            return false;
        }
        self.value.pop();
        true
    }

    /// Get display string (padded or with placeholders)
    pub fn display_string(&self) -> String {
        let len = self.segment_type.length();
        if self.value.is_empty() {
            // Show placeholder
            match self.segment_type {
                SegmentType::Year => "YYYY".to_string(),
                SegmentType::Month => "MM".to_string(),
                SegmentType::Day => "DD".to_string(),
                SegmentType::Hour => "HH".to_string(),
                SegmentType::Minute => "mm".to_string(),
                SegmentType::Second => "ss".to_string(),
            }
        } else if self.value.len() < len {
            // Partial input - show as is with underscores
            format!("{}{}", self.value, "_".repeat(len - self.value.len()))
        } else {
            // Full value
            self.value.clone()
        }
    }
}

/// Date input node - structured date/time input with segments
pub struct DateInputNode {
    pub id: NodeId,
    pub label: String,
    pub segments: Vec<DateSegment>,
    pub separators: Vec<String>, // Separators between segments (e.g., "/", ":", "-")
    pub focused_segment: usize,  // Which segment is currently focused
    pub validators: Vec<Validator>,
    pub error: Option<String>,
    pub min_width: usize,
}

impl DateInputNode {
    /// Create a new DateInput from a format string
    /// Format examples: "DD/MM/YYYY", "YYYY-MM-DD", "HH:mm:ss", "DD/MM/YYYY HH:mm"
    pub fn new(id: impl Into<NodeId>, label: impl Into<String>, format: &str) -> Self {
        let (segments, separators) = Self::parse_format(format);

        Self {
            id: id.into(),
            label: label.into(),
            segments,
            separators,
            focused_segment: 0,
            validators: Vec::new(),
            error: None,
            min_width: 10,
        }
    }

    /// Parse format string into segments and separators
    fn parse_format(format: &str) -> (Vec<DateSegment>, Vec<String>) {
        let mut segments = Vec::new();
        let mut separators = Vec::new();
        let mut current_token = String::new();
        let mut current_separator = String::new();

        for ch in format.chars() {
            if ch.is_alphabetic() {
                if !current_separator.is_empty() {
                    separators.push(current_separator.clone());
                    current_separator.clear();
                }
                current_token.push(ch);
            } else {
                if !current_token.is_empty() {
                    if let Some(seg_type) = SegmentType::from_token(&current_token) {
                        segments.push(DateSegment::new(seg_type));
                    }
                    current_token.clear();
                }
                current_separator.push(ch);
            }
        }

        // Handle last token
        if !current_token.is_empty() {
            if let Some(seg_type) = SegmentType::from_token(&current_token) {
                segments.push(DateSegment::new(seg_type));
            }
        }

        // Add trailing separator if exists
        if !current_separator.is_empty() {
            separators.push(current_separator);
        }

        // Ensure we have n-1 separators for n segments (pad with empty if needed)
        while separators.len() < segments.len().saturating_sub(1) {
            separators.push(String::new());
        }

        (segments, separators)
    }

    /// Get the full value as a formatted string
    pub fn value(&self) -> String {
        let mut result = String::new();
        for (i, segment) in self.segments.iter().enumerate() {
            result.push_str(&segment.value);
            if i < self.separators.len() {
                result.push_str(&self.separators[i]);
            }
        }
        result
    }

    /// Get display string with separators and placeholders
    pub fn display_string(&self) -> String {
        let mut result = String::new();
        for (i, segment) in self.segments.iter().enumerate() {
            result.push_str(&segment.display_string());
            if i < self.separators.len() {
                result.push_str(&self.separators[i]);
            }
        }
        result
    }

    /// Validate all segments
    pub fn validate(&self) -> Result<(), String> {
        // Check that all segments are filled
        for segment in &self.segments {
            if segment.value.is_empty() {
                return Err("All fields must be filled".to_string());
            }
            let val = segment.numeric_value();
            let min = segment.segment_type.min_value();
            let max = segment.segment_type.max_value();
            if val < min || val > max {
                return Err(format!("Invalid value for {:?}", segment.segment_type));
            }
        }

        // Run custom validators
        let value_str = self.value();
        for validator in &self.validators {
            validator(&value_str)?;
        }

        Ok(())
    }

    /// Move focus to next segment
    pub fn move_next(&mut self) -> bool {
        if self.focused_segment < self.segments.len().saturating_sub(1) {
            self.focused_segment += 1;
            true
        } else {
            false
        }
    }

    /// Move focus to previous segment
    pub fn move_prev(&mut self) -> bool {
        if self.focused_segment > 0 {
            self.focused_segment -= 1;
            true
        } else {
            false
        }
    }

    /// Increment current segment value
    pub fn increment(&mut self) -> bool {
        if let Some(segment) = self.segments.get_mut(self.focused_segment) {
            segment.increment();
            true
        } else {
            false
        }
    }

    /// Decrement current segment value
    pub fn decrement(&mut self) -> bool {
        if let Some(segment) = self.segments.get_mut(self.focused_segment) {
            segment.decrement();
            true
        } else {
            false
        }
    }

    /// Insert a digit into the current segment
    pub fn insert_digit(&mut self, digit: char) -> bool {
        if let Some(segment) = self.segments.get_mut(self.focused_segment) {
            let result = segment.insert_digit(digit);

            // Auto-advance if segment is full
            if result && segment.value.len() == segment.segment_type.length() {
                self.move_next();
            }

            result
        } else {
            false
        }
    }

    /// Delete a digit from the current segment
    pub fn delete_digit(&mut self) -> bool {
        if let Some(segment) = self.segments.get_mut(self.focused_segment) {
            segment.delete_digit()
        } else {
            false
        }
    }

    /// Clear all segments
    pub fn clear(&mut self) {
        for segment in &mut self.segments {
            segment.value.clear();
        }
        self.focused_segment = 0;
    }
}

/// Builder for DateInput node
pub struct DateInputBuilder {
    input: DateInputNode,
    opts: Options,
}

impl DateInputBuilder {
    pub fn new(id: NodeId, label: String, format: String) -> Self {
        Self {
            input: DateInputNode::new(id, label, &format),
            opts: Options::default(),
        }
    }

    pub fn min_width(mut self, width: usize) -> Self {
        self.input.min_width = width;
        self
    }

    pub fn validator(mut self, validator: Validator) -> Self {
        self.input.validators.push(validator);
        self
    }

    // Display options

    pub fn with_display(mut self, display: Display) -> Self {
        self.opts.display = display;
        self
    }

    pub fn with_wrap(mut self, wrap: Wrap) -> Self {
        self.opts.wrap = wrap;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.opts.color = Some(color);
        self
    }

    pub fn with_background(mut self, bg: Color) -> Self {
        self.opts.background = Some(bg);
        self
    }

    pub fn bold(mut self) -> Self {
        self.opts.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.opts.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.opts.underline = true;
        self
    }

    // Build

    pub fn build(self) -> Node {
        Node {
            opts: self.opts,
            kind: NodeKind::DateInput(DateInputNode {
                id: self.input.id,
                label: self.input.label,
                segments: self.input.segments,
                separators: self.input.separators,
                focused_segment: self.input.focused_segment,
                validators: self.input.validators,
                error: self.input.error,
                min_width: self.input.min_width,
            }),
        }
    }
}

// Auto-build when used in Vec<Node>
impl From<DateInputBuilder> for Node {
    fn from(builder: DateInputBuilder) -> Self {
        builder.build()
    }
}
