#[derive(Debug, Clone, Default)]
pub struct HelpDisplayCtx {
    name: String,

    head: String,

    foot: String,

    width: usize,

    usagew: usize,

    subnames: Vec<String>,

    submode: bool,
}

impl HelpDisplayCtx {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_head(mut self, head: impl Into<String>) -> Self {
        self.head = head.into();
        self
    }

    pub fn with_foot(mut self, foot: impl Into<String>) -> Self {
        self.foot = foot.into();
        self
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn with_usagew(mut self, usagew: usize) -> Self {
        self.usagew = usagew;
        self
    }

    pub fn with_subnames(mut self, subnames: Vec<String>) -> Self {
        self.subnames = subnames;
        self
    }

    pub fn with_submode(mut self, submode: bool) -> Self {
        self.submode = submode;
        self
    }

    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn set_head(&mut self, head: impl Into<String>) -> &mut Self {
        self.head = head.into();
        self
    }

    pub fn set_foot(&mut self, foot: impl Into<String>) -> &mut Self {
        self.foot = foot.into();
        self
    }

    pub fn set_width(&mut self, width: usize) -> &mut Self {
        self.width = width;
        self
    }

    pub fn set_usagew(&mut self, usagew: usize) -> &mut Self {
        self.usagew = usagew;
        self
    }

    pub fn set_subnames(&mut self, subnames: Vec<String>) -> &mut Self {
        self.subnames = subnames;
        self
    }

    pub fn set_submode(&mut self, submode: bool) -> &mut Self {
        self.submode = submode;
        self
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn head(&self) -> &String {
        &self.head
    }

    pub fn foot(&self) -> &String {
        &self.foot
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn usagew(&self) -> usize {
        self.usagew
    }

    pub fn subnames(&self) -> &[String] {
        &self.subnames
    }

    pub fn submode(&self) -> bool {
        self.submode
    }

    pub fn generate_name(&self) -> String {
        if self.submode {
            std::iter::once(self.name())
                .chain(self.subnames().iter())
                .map(|v| v.as_str())
                .collect::<Vec<&str>>()
                .join(" ")
        } else {
            self.subnames()
                .iter()
                .chain(std::iter::once(self.name()))
                .map(|v| v.as_str())
                .collect::<Vec<&str>>()
                .join(" ")
        }
    }
}
