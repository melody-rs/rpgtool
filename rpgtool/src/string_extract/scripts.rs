use super::GameString;

#[cfg(feature = "ruby-prism")]
struct ScriptVisitor<'a> {
    strings: &'a mut Vec<GameString>,
    context: String,
}

// TODO handle unwraps
// FIXME better way to indentify strings
#[cfg(feature = "ruby-prism")]
#[allow(clippy::unwrap_used)]
impl<'pr> ruby_prism::Visit<'pr> for ScriptVisitor<'_> {
    fn visit_string_node(&mut self, node: &ruby_prism::StringNode<'pr>) {
        let location = format!("{}:{}", self.context, node.location().start_offset());
        let text = str::from_utf8(node.content_loc().as_slice())
            .unwrap()
            .to_owned();
        self.strings.push(GameString { location, text });
    }

    fn visit_x_string_node(&mut self, node: &ruby_prism::XStringNode<'pr>) {
        let location = format!("{}:{}", self.context, node.location().start_offset());
        let text = str::from_utf8(node.content_loc().as_slice())
            .unwrap()
            .to_owned();
        self.strings.push(GameString { location, text });
    }

    fn visit_interpolated_string_node(&mut self, node: &ruby_prism::InterpolatedStringNode<'pr>) {
        let location = format!("{}:{}", self.context, node.location().start_offset());

        let literal = node.location().as_slice();
        let content = &literal[..literal.len() - 1][1..];
        // ?? how to handle interpolation?
        let text = str::from_utf8(content).unwrap().to_owned();
        self.strings.push(GameString { location, text });
    }

    fn visit_interpolated_x_string_node(
        &mut self,
        node: &ruby_prism::InterpolatedXStringNode<'pr>,
    ) {
        let location = format!("{}:{}", self.context, node.location().start_offset());

        let literal = node.location().as_slice();
        let content = &literal[..literal.len() - 1][1..];
        // ?? how to handle interpolation?
        let text = str::from_utf8(content).unwrap().to_owned();
        self.strings.push(GameString { location, text });
    }
}

#[allow(clippy::needless_pass_by_value)] // is used by value when prism is disabled
pub fn process_script_text(text: String, strings: &mut Vec<GameString>, context: String) {
    #[cfg(feature = "ruby-prism")]
    {
        use ruby_prism::Visit;

        let mut visitor = ScriptVisitor { strings, context };
        let parsed = ruby_prism::parse(text.as_bytes());
        visitor.visit(&parsed.node());
    }
    #[cfg(not(feature = "ruby-prism"))]
    {
        strings.push(GameString {
            location: context,
            text,
        });
    }
}
