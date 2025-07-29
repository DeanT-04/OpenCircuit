//! Chat handler for AI interactions in OpenCircuit
//! 
//! This module manages the chat conversation flow, message processing,
//! and integration with AI services for circuit design assistance.

use crate::gui::ChatMessage;
use crate::OpenCircuitResult;
use chrono::Utc;
use std::collections::VecDeque;
use tokio::time::{sleep, Duration};

/// Maximum number of messages to keep in conversation history
const MAX_CONVERSATION_HISTORY: usize = 50;

/// Chat handler for managing AI conversations
pub struct ChatHandler {
    /// Conversation history for context
    conversation_history: VecDeque<ChatMessage>,
    /// System prompt for the AI assistant
    system_prompt: String,
    /// Whether the handler is currently processing a request
    is_processing: bool,
}

impl Default for ChatHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatHandler {
    pub fn new() -> Self {
        Self {
            conversation_history: VecDeque::new(),
            system_prompt: Self::default_system_prompt(),
            is_processing: false,
        }
    }

    /// Get the default system prompt for the AI assistant
    fn default_system_prompt() -> String {
        r#"You are an expert AI assistant for OpenCircuit, a circuit design and PCB layout tool.

Your expertise includes:
- Electronic circuit design and analysis
- Component selection and specifications
- PCB layout best practices
- Circuit simulation and troubleshooting
- Power supply design
- Signal integrity considerations
- EMC/EMI design guidelines
- Manufacturing considerations

Guidelines for responses:
- Be helpful, accurate, and educational
- Provide specific technical details when appropriate
- Suggest component values and part numbers when possible
- Consider practical constraints like cost, availability, and manufacturability
- Ask clarifying questions when requirements are unclear
- Use appropriate technical terminology but explain complex concepts
- Reference industry standards and best practices

Always aim to help users create better, more reliable circuit designs."#.to_string()
    }

    /// Add a message to the conversation history
    pub fn add_message(&mut self, message: ChatMessage) {
        self.conversation_history.push_back(message);
        
        // Trim history if it gets too long
        while self.conversation_history.len() > MAX_CONVERSATION_HISTORY {
            self.conversation_history.pop_front();
        }
    }

    /// Process a user message and generate an AI response
    pub async fn process_message(&mut self, user_message: &str) -> OpenCircuitResult<ChatMessage> {
        if self.is_processing {
            return Ok(ChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                content: "I'm still processing your previous message. Please wait a moment.".to_string(),
                is_user: false,
                timestamp: Utc::now(),
            });
        }

        self.is_processing = true;

        // Add user message to history
        let user_msg = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            content: user_message.to_string(),
            is_user: true,
            timestamp: Utc::now(),
        };
        self.add_message(user_msg);

        // Simulate processing delay (will be replaced with actual API call)
        sleep(Duration::from_millis(500)).await;

        // Generate response based on message content and context
        let response_content = self.generate_contextual_response(user_message).await?;

        let ai_response = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            content: response_content,
            is_user: false,
            timestamp: Utc::now(),
        };

        self.add_message(ai_response.clone());
        self.is_processing = false;

        Ok(ai_response)
    }

    /// Generate a contextual response based on the user's message and conversation history
    async fn generate_contextual_response(&self, user_message: &str) -> OpenCircuitResult<String> {
        let message_lower = user_message.to_lowercase();
        
        // Analyze message for circuit design topics
        if self.contains_circuit_keywords(&message_lower) {
            self.generate_circuit_response(&message_lower).await
        } else if self.contains_component_keywords(&message_lower) {
            self.generate_component_response(&message_lower).await
        } else if self.contains_pcb_keywords(&message_lower) {
            self.generate_pcb_response(&message_lower).await
        } else if self.contains_simulation_keywords(&message_lower) {
            self.generate_simulation_response(&message_lower).await
        } else if self.is_greeting(&message_lower) {
            Ok(self.generate_greeting_response())
        } else {
            self.generate_general_response(user_message).await
        }
    }

    fn contains_circuit_keywords(&self, message: &str) -> bool {
        let keywords = ["circuit", "schematic", "design", "topology", "amplifier", "filter", "oscillator"];
        keywords.iter().any(|&keyword| message.contains(keyword))
    }

    fn contains_component_keywords(&self, message: &str) -> bool {
        let keywords = ["resistor", "capacitor", "inductor", "transistor", "diode", "ic", "component", "part"];
        keywords.iter().any(|&keyword| message.contains(keyword))
    }

    fn contains_pcb_keywords(&self, message: &str) -> bool {
        let keywords = ["pcb", "layout", "routing", "trace", "via", "layer", "stackup", "placement"];
        keywords.iter().any(|&keyword| message.contains(keyword))
    }

    fn contains_simulation_keywords(&self, message: &str) -> bool {
        let keywords = ["simulation", "spice", "analysis", "frequency", "transient", "dc", "ac"];
        keywords.iter().any(|&keyword| message.contains(keyword))
    }

    fn is_greeting(&self, message: &str) -> bool {
        let greetings = ["hello", "hi", "hey", "good morning", "good afternoon", "good evening"];
        greetings.iter().any(|&greeting| message.contains(greeting))
    }

    async fn generate_circuit_response(&self, message: &str) -> OpenCircuitResult<String> {
        if message.contains("amplifier") {
            Ok("🔊 For amplifier design, key considerations include:\n\n• Gain requirements and bandwidth\n• Input/output impedance matching\n• Power supply voltage and current\n• Noise and distortion specifications\n• Stability and compensation\n\nWhat type of amplifier are you designing? (op-amp, discrete, RF, audio, etc.)".to_string())
        } else if message.contains("filter") {
            Ok("🔧 Filter design depends on your requirements:\n\n• Low-pass, high-pass, band-pass, or band-stop?\n• Cutoff frequency and roll-off rate\n• Passband ripple and stopband attenuation\n• Active vs passive implementation\n• Component tolerance effects\n\nCould you specify your filter requirements?".to_string())
        } else if message.contains("oscillator") {
            Ok("⚡ Oscillator design involves several key factors:\n\n• Frequency stability and accuracy\n• Phase noise requirements\n• Output power and waveform\n• Temperature and supply sensitivity\n• Crystal vs RC vs LC topology\n\nWhat frequency range and stability do you need?".to_string())
        } else {
            Ok("🔌 I'd be happy to help with your circuit design! Could you provide more details about:\n\n• The intended function or application\n• Input/output requirements\n• Power supply constraints\n• Performance specifications\n• Any specific design challenges\n\nThis will help me give you more targeted advice.".to_string())
        }
    }

    async fn generate_component_response(&self, message: &str) -> OpenCircuitResult<String> {
        if message.contains("resistor") {
            Ok("🔧 Resistor selection considerations:\n\n• Resistance value and tolerance (1%, 5%, etc.)\n• Power rating (1/8W, 1/4W, 1/2W, etc.)\n• Temperature coefficient (ppm/°C)\n• Package type (0603, 0805, through-hole)\n• Special types (precision, high-power, current sense)\n\nWhat's your target resistance and power requirement?".to_string())
        } else if message.contains("capacitor") {
            Ok("⚡ Capacitor selection guide:\n\n• Capacitance value and tolerance\n• Voltage rating (with derating factor)\n• Dielectric type (X7R, C0G, Y5V for ceramics)\n• ESR and ESL for high-frequency applications\n• Temperature stability and aging\n• Package size and mounting\n\nWhat's your application? (decoupling, timing, energy storage, etc.)".to_string())
        } else if message.contains("transistor") {
            Ok("🔌 Transistor selection criteria:\n\n• BJT vs MOSFET vs JFET\n• Voltage and current ratings\n• Switching speed and frequency response\n• Power dissipation and thermal considerations\n• Gain characteristics (hFE, gm)\n• Package type and pinout\n\nWhat's your application? (switching, amplification, etc.)".to_string())
        } else {
            Ok("🧩 Component selection is crucial for reliable designs. Key factors include:\n\n• Electrical specifications (voltage, current, frequency)\n• Environmental conditions (temperature, humidity)\n• Mechanical constraints (size, mounting)\n• Cost and availability\n• Reliability and lifetime\n\nWhich specific component are you looking for?".to_string())
        }
    }

    async fn generate_pcb_response(&self, message: &str) -> OpenCircuitResult<String> {
        if message.contains("layout") || message.contains("placement") {
            Ok("📋 PCB layout best practices:\n\n• Component placement for signal flow\n• Minimize trace lengths for high-speed signals\n• Proper ground plane design\n• Thermal considerations and heat dissipation\n• Manufacturing constraints (drill sizes, trace width)\n• EMI/EMC considerations\n\nWhat type of circuit are you laying out?".to_string())
        } else if message.contains("routing") || message.contains("trace") {
            Ok("🛤️ PCB routing guidelines:\n\n• Trace width for current carrying capacity\n• Differential pair routing for high-speed signals\n• Via placement and stitching\n• Layer stackup and impedance control\n• Crosstalk minimization\n• Return path continuity\n\nWhat's your signal frequency and current requirements?".to_string())
        } else {
            Ok("🔧 PCB design involves many considerations:\n\n• Layer count and stackup\n• Component placement strategy\n• Signal integrity requirements\n• Power distribution network\n• Thermal management\n• Manufacturing and assembly constraints\n\nWhat specific PCB design challenge can I help with?".to_string())
        }
    }

    async fn generate_simulation_response(&self, message: &str) -> OpenCircuitResult<String> {
        Ok("📊 Circuit simulation is essential for design verification:\n\n• DC operating point analysis\n• AC frequency response\n• Transient time-domain analysis\n• Monte Carlo tolerance analysis\n• Temperature and process variations\n• Worst-case design verification\n\nWhat type of analysis do you need for your circuit? I can help you set up the appropriate simulation parameters.".to_string())
    }

    fn generate_greeting_response(&self) -> String {
        "👋 Hello! I'm your OpenCircuit AI assistant, ready to help with all your electronics design needs!\n\n🔧 I can assist with:\n• Circuit design and analysis\n• Component selection and specifications\n• PCB layout optimization\n• Simulation setup and interpretation\n• Design troubleshooting\n\nWhat electronics project are you working on today?".to_string()
    }

    async fn generate_general_response(&self, user_message: &str) -> OpenCircuitResult<String> {
        Ok(format!("🤖 I understand you're asking about: \"{}\"\n\nI'm here to help with electronics and circuit design. Could you provide more context about your project or question? For example:\n\n• What type of circuit are you working on?\n• What specific challenge are you facing?\n• Are you looking for component recommendations?\n• Do you need help with PCB layout?\n\nThe more details you provide, the better I can assist you!", user_message))
    }

    /// Get conversation history
    pub fn get_conversation_history(&self) -> &VecDeque<ChatMessage> {
        &self.conversation_history
    }

    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.conversation_history.clear();
    }

    /// Check if currently processing a message
    pub fn is_processing(&self) -> bool {
        self.is_processing
    }

    /// Update system prompt
    pub fn set_system_prompt(&mut self, prompt: String) {
        self.system_prompt = prompt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_handler_creation() {
        let handler = ChatHandler::new();
        assert!(!handler.is_processing());
        assert_eq!(handler.get_conversation_history().len(), 0);
    }

    #[test]
    fn test_keyword_detection() {
        let handler = ChatHandler::new();
        
        assert!(handler.contains_circuit_keywords("I need help with my amplifier circuit"));
        assert!(handler.contains_component_keywords("What resistor should I use?"));
        assert!(handler.contains_pcb_keywords("How do I route this trace?"));
        assert!(handler.is_greeting("Hello there!"));
    }

    #[tokio::test]
    async fn test_message_processing() {
        let mut handler = ChatHandler::new();
        let response = handler.process_message("Hello").await.unwrap();
        
        assert!(!response.is_user);
        assert!(response.content.contains("Hello"));
        assert_eq!(handler.get_conversation_history().len(), 2); // User + AI message
    }
}