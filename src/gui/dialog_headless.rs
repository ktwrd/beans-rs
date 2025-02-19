pub struct DialogBuilder
{
    pub title: String,
    pub content: String,
}

pub enum DialogIconKind
{
    Default,
    Warn,
    Error
}

impl Default for crate::gui::DialogBuilder
{
    fn default() -> Self
    {
        Self {
            title: format!("beans v{}", crate::VERSION),
            content: String::new()
        }
    }
}

impl crate::gui::DialogBuilder
{
    pub fn new() -> Self
    {
        Self::default()
    }
    pub fn with_png_data(
        mut self,
        data: &[u8]
    ) -> Self
    {
        self
    }
    pub fn with_icon(
        self,
        kind: crate::gui::DialogIconKind
    ) -> Self
    {
        self
    }
    pub fn with_title(
        mut self,
        content: String
    ) -> Self
    {
        self.title = content.clone();
        self
    }
    pub fn with_content(
        mut self,
        content: String
    ) -> Self
    {
        self.content = content.clone();
        self
    }
    pub fn run(&self)
    {
        println!("============ {} ============", self.title);
        println!("{}", self.content);
    }
}