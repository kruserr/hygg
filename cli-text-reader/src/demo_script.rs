use crossterm::event::{KeyCode, KeyModifiers};
use std::time::Duration;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum DemoAction {
    // Wait for a duration
    Wait(Duration),
    // Press a single key
    Key(KeyCode),
    // Press a key with modifiers
    KeyWithModifiers(KeyCode, KeyModifiers),
    // Type a string character by character
    TypeString(String, Duration), // string, delay between chars
    // Show a subtitle/hint
    ShowHint(String, Duration), // hint text, display duration
    // Mark a checkpoint (for debugging)
    Checkpoint(String),
    // Execute a vim motion (e.g., "vip", "viw", "yy")
    VimMotion(String),
}

#[allow(dead_code)]
pub struct DemoScript {
    pub actions: Vec<DemoAction>,
    pub total_duration: Duration,
}

impl DemoScript {
    pub fn marketing_demo() -> Self {
        use DemoAction::*;
        
        let actions = vec![
            Wait(Duration::from_millis(2000)),

            ShowHint("select entire paragraphs\nwith a single command".to_string(), Duration::from_millis(3500)),
            Wait(Duration::from_millis(500)),
            VimMotion("vip".to_string()),
            Wait(Duration::from_millis(2000)),
            
            ShowHint("highlight selected text".to_string(), Duration::from_millis(3500)),
            Wait(Duration::from_millis(500)),
            Key(KeyCode::Char(':')),
            Wait(Duration::from_millis(250)),
            Key(KeyCode::Char('h')),
            Wait(Duration::from_millis(500)),
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(1500)),
            
            ShowHint("execute any command\ndirectly from your reader".to_string(), Duration::from_millis(3500)),
            Wait(Duration::from_millis(500)),
            Key(KeyCode::Char(':')),
            Wait(Duration::from_millis(250)),
            Key(KeyCode::Char('!')),
            Wait(Duration::from_millis(250)),
            Key(KeyCode::Char('l')),
            Wait(Duration::from_millis(250)),
            Key(KeyCode::Char('s')),
            Wait(Duration::from_millis(500)),
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(1000)),
            
            Key(KeyCode::Char('j')),
            Wait(Duration::from_millis(400)),
            Key(KeyCode::Char('j')),
            Wait(Duration::from_millis(400)),
            Key(KeyCode::Char('j')),
            Wait(Duration::from_millis(1000)),
            
            ShowHint("copy command output".to_string(), Duration::from_millis(3500)),
            Wait(Duration::from_millis(500)),
            Key(KeyCode::Char('y')),
            Wait(Duration::from_millis(250)),
            Key(KeyCode::Char('y')),
            Wait(Duration::from_millis(1000)),
            
            ShowHint("paste output into\nyour next command".to_string(), Duration::from_millis(3500)),
            Wait(Duration::from_millis(500)),
            Key(KeyCode::Char(':')),
            Wait(Duration::from_millis(250)),
            Key(KeyCode::Char('!')),
            Wait(Duration::from_millis(250)),
            Key(KeyCode::Char('c')),
            Wait(Duration::from_millis(250)),
            Key(KeyCode::Char('a')),
            Wait(Duration::from_millis(250)),
            Key(KeyCode::Char('t')),
            Wait(Duration::from_millis(250)),
            Key(KeyCode::Char(' ')),
            Wait(Duration::from_millis(250)),
            KeyWithModifiers(KeyCode::Char('v'), KeyModifiers::CONTROL),
            Wait(Duration::from_millis(500)),
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(2000)),

            ShowHint("hygg - simplifying the way you read\n\nTransform your terminal into a powerful document reader\n\ngithub.com/kruserr/hygg".to_string(), Duration::from_millis(5000)),
            Wait(Duration::from_millis(4000)),
        ];
        
        Self {
            actions,
            total_duration: Duration::from_secs(35),
        }
    }
    
    #[allow(dead_code)]
    pub fn beginner_tutorial() -> Self {
        use DemoAction::*;
        
        let actions = vec![
            // Welcome
            ShowHint("Welcome to hygg! Let's learn the basics.".to_string(), Duration::from_millis(3000)),
            Wait(Duration::from_millis(1000)),
            
            // Basic movement
            ShowHint("Use j/k to move down/up (try it now!)".to_string(), Duration::from_millis(5000)),
            Checkpoint("Waiting for user to practice j/k".to_string()),
            
            // More to be added based on user interaction
        ];
        
        Self {
            actions,
            total_duration: Duration::from_secs(60), // Much longer for learning
        }
    }
    
    // Test script for the interactive tutorial
    #[allow(dead_code)]
    pub fn tutorial_test() -> Self {
        use DemoAction::*;
        
        let actions = vec![
            // Wait for initial load
            Wait(Duration::from_millis(500)),
            
            // Step 1: Welcome - press 'j' to move down
            Wait(Duration::from_millis(500)),
            Key(KeyCode::Char('j')),
            Wait(Duration::from_millis(500)),
            
            // Type :next to continue
            Key(KeyCode::Char(':')),
            Wait(Duration::from_millis(100)),
            Key(KeyCode::Char('n')),
            Key(KeyCode::Char('e')),
            Key(KeyCode::Char('x')),
            Key(KeyCode::Char('t')),
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(500)),
            
            // Step 2: Visual Selection - press 'v' and move to select, then 'y' to yank
            Key(KeyCode::Char('v')),
            Wait(Duration::from_millis(200)),
            Key(KeyCode::Char('w')),
            Wait(Duration::from_millis(200)),
            Key(KeyCode::Char('y')),
            Wait(Duration::from_millis(500)),
            
            // Type :next
            Key(KeyCode::Char(':')),
            Wait(Duration::from_millis(100)),
            Key(KeyCode::Char('n')),
            Key(KeyCode::Char('e')),
            Key(KeyCode::Char('x')),
            Key(KeyCode::Char('t')),
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(500)),
            
            // Step 3: Text Objects - vip to select paragraph, then :h to highlight
            VimMotion("vip".to_string()),
            Wait(Duration::from_millis(500)),
            Key(KeyCode::Char(':')),
            Wait(Duration::from_millis(100)),
            Key(KeyCode::Char('h')),
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(500)),
            
            // Type :next
            Key(KeyCode::Char(':')),
            Wait(Duration::from_millis(100)),
            Key(KeyCode::Char('n')),
            Key(KeyCode::Char('e')),
            Key(KeyCode::Char('x')),
            Key(KeyCode::Char('t')),
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(500)),
            
            // Step 4: Search - search for "special"
            Key(KeyCode::Char('/')),
            Wait(Duration::from_millis(200)),
            Key(KeyCode::Char('s')),
            Key(KeyCode::Char('p')),
            Key(KeyCode::Char('e')),
            Key(KeyCode::Char('c')),
            Key(KeyCode::Char('i')),
            Key(KeyCode::Char('a')),
            Key(KeyCode::Char('l')),
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(500)),
            
            // Type :next
            Key(KeyCode::Char(':')),
            Wait(Duration::from_millis(100)),
            Key(KeyCode::Char('n')),
            Key(KeyCode::Char('e')),
            Key(KeyCode::Char('x')),
            Key(KeyCode::Char('t')),
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(500)),
            
            // Step 5: Bookmarks - set bookmark 'a' with 'ma'
            Key(KeyCode::Char('m')),
            Wait(Duration::from_millis(200)),
            Key(KeyCode::Char('a')),
            Wait(Duration::from_millis(500)),
            
            // Type :next
            Key(KeyCode::Char(':')),
            Wait(Duration::from_millis(100)),
            Key(KeyCode::Char('n')),
            Key(KeyCode::Char('e')),
            Key(KeyCode::Char('x')),
            Key(KeyCode::Char('t')),
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(500)),
            
            // Step 6: Command Execution - run :!echo hello world
            Key(KeyCode::Char(':')),
            Wait(Duration::from_millis(100)),
            Key(KeyCode::Char('!')),
            Key(KeyCode::Char('e')),
            Key(KeyCode::Char('c')),
            Key(KeyCode::Char('h')),
            Key(KeyCode::Char('o')),
            Key(KeyCode::Char(' ')),
            Key(KeyCode::Char('h')),
            Key(KeyCode::Char('e')),
            Key(KeyCode::Char('l')),
            Key(KeyCode::Char('l')),
            Key(KeyCode::Char('o')),
            Key(KeyCode::Char(' ')),
            Key(KeyCode::Char('w')),
            Key(KeyCode::Char('o')),
            Key(KeyCode::Char('r')),
            Key(KeyCode::Char('l')),
            Key(KeyCode::Char('d')),
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(1000)), // Wait for command execution
            
            // Type :next
            Key(KeyCode::Char(':')),
            Wait(Duration::from_millis(100)),
            Key(KeyCode::Char('n')),
            Key(KeyCode::Char('e')),
            Key(KeyCode::Char('x')),
            Key(KeyCode::Char('t')),
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(500)),
            
            // Final step: Press Enter to complete tutorial
            Key(KeyCode::Enter),
            Wait(Duration::from_millis(1000)),
        ];
        
        Self {
            actions,
            total_duration: Duration::from_secs(20),
        }
    }
}