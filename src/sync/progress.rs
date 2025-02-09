use console::{style, Emoji};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub struct SyncProgress {
    max_tool_size: usize,
    max_tag_size: usize,
    multi_progress: MultiProgress,
}

const SUCCESS: Emoji<'_, '_> = Emoji("✅  ", "OK ");
const FAILURE: Emoji<'_, '_> = Emoji("⛔  ", "NO ");
const PROCESS: Emoji<'_, '_> = Emoji("📥  ", ".. ");
const MIN_TAG_SIZE: usize = 8;

impl SyncProgress {
    /// Creates new `SyncProgress` from a list of tools.
    /// !!! The given `Vec` must be non-empty !!!
    pub fn new(tools: Vec<String>, tags: Vec<String>) -> SyncProgress {
        // unwrap is safe here because 'new' is called with a non-empty vector
        let max_tool_size = tools.iter().map(|tool| tool.len()).max().unwrap();

        // putting a default of 8 here since tags like v0.10.10 is already 8
        let max_tag_size = tags
            .iter()
            .map(|tag| std::cmp::max(tag.len(), MIN_TAG_SIZE))
            .max()
            .unwrap_or(MIN_TAG_SIZE);

        let multi_progress = MultiProgress::new();

        SyncProgress {
            max_tool_size,
            max_tag_size,
            multi_progress,
        }
    }

    fn fmt_prefix(&self, emoji: Emoji, tool_name: &str, tag_name: &str) -> String {
        let aligned_tool = format!(
            "{:tool_width$} {:tag_width$}",
            tool_name,
            tag_name,
            tool_width = self.max_tool_size,
            tag_width = self.max_tag_size,
        );

        format!("{}{}", emoji, aligned_tool)
    }

    pub fn create_message_bar(&self, tool_name: &str, tag_name: &str) -> ProgressBar {
        let message_style = ProgressStyle::with_template("{prefix:.bold.dim} {msg}").unwrap();

        self.multi_progress.add(
            ProgressBar::new(100)
                .with_style(message_style)
                .with_prefix(self.fmt_prefix(PROCESS, tool_name, tag_name)),
        )
    }

    pub fn create_progress_bar(&self, size: u64) -> ProgressBar {
        let bar_style =
            ProgressStyle::with_template("{bytes}/{total_bytes} {wide_bar:.cyan/blue}").unwrap();

        self.multi_progress
            .add(ProgressBar::new(size).with_style(bar_style))
    }

    pub fn finish_progress(pb: ProgressBar) {
        pb.finish_and_clear()
    }

    pub fn success(&self, pb: ProgressBar, tool_name: &str, tag_name: &str) {
        pb.set_prefix(self.fmt_prefix(SUCCESS, tool_name, tag_name));

        let success_msg = format!("{}", style("Completed!").bold().green());
        pb.set_message(success_msg);
        pb.finish();
    }

    pub fn failure(&self, pb: ProgressBar, tool_name: &str, tag_name: &str, err_msg: String) {
        pb.set_prefix(self.fmt_prefix(FAILURE, tool_name, tag_name));

        let failure_msg = format!("{}", style(err_msg).red());
        pb.set_message(failure_msg);
        pb.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::SyncProgress;

    #[test]
    fn test_max_tag_size_specific() {
        let tags: Vec<String> = vec![
            String::from("v10.10.100"),
            String::from("latest"),
            String::from("latest"),
        ];
        let tools: Vec<String> = vec![
            String::from("ripgrep"),
            String::from("bat"),
            String::from("exa"),
        ];

        let progres = SyncProgress::new(tools, tags);

        // v10.10.100 is 10 characters
        assert_eq!(progres.max_tag_size, 10);
        // ripgrep is 7 characters
        assert_eq!(progres.max_tool_size, 7);
    }

    #[test]
    fn test_max_tag_size_latest() {
        let tags: Vec<String> = vec![
            String::from("latest"),
            String::from("latest"),
            String::from("latest"),
        ];
        let tools: Vec<String> = vec![
            String::from("ripgrep"),
            String::from("bat"),
            String::from("exa"),
        ];

        let progres = SyncProgress::new(tools, tags);

        // latest is 6 characters so it should default to 8
        assert_eq!(progres.max_tag_size, 8);
        // ripgrep is 7 characters
        assert_eq!(progres.max_tool_size, 7);
    }
}
