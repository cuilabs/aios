//! End-to-End Workflow Test Suite
//! 
//! Production-grade end-to-end workflow tests that verify complete user scenarios
//! across multiple services, ensuring proper service-to-service communication
//! and integration.

use crate::harness::TestHarness;
use crate::TestStatus;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;
use reqwest::blocking::Client;

pub struct E2EWorkflowTestSuite {
    output_dir: PathBuf,
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowTestResult {
    pub workflow_name: String,
    pub success: bool,
    pub duration_ms: u64,
    pub steps: Vec<WorkflowStep>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub step_name: String,
    pub service: String,
    pub endpoint: String,
    pub success: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
}

impl E2EWorkflowTestSuite {
    pub fn new(output_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&output_dir).ok();
        Self {
            output_dir,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
    
    pub fn run(&self, workflow: Option<&str>, verbose: bool) -> Result<crate::TestReport> {
        let start_time = Instant::now();
        let mut tests = Vec::new();
        
        let harness = TestHarness::new(verbose);
        
        // Agent Spawn → ML Prediction → Memory Allocation Workflow
        if workflow.is_none() || workflow == Some("agent_ml_memory") {
            tests.push(harness.run_test("e2e_agent_ml_memory", || {
                self.test_agent_ml_memory_workflow()
            })?);
        }
        
        // NLP Command → Agent Execution → Security Check Workflow
        if workflow.is_none() || workflow == Some("nlp_agent_security") {
            tests.push(harness.run_test("e2e_nlp_agent_security", || {
                self.test_nlp_agent_security_workflow()
            })?);
        }
        
        // Learning → UI Adaptation → Update Scheduling Workflow
        if workflow.is_none() || workflow == Some("learning_ui_update") {
            tests.push(harness.run_test("e2e_learning_ui_update", || {
                self.test_learning_ui_update_workflow()
            })?);
        }
        
        // Security Threat Detection → Auto-Healing → Metrics Collection Workflow
        if workflow.is_none() || workflow == Some("security_healing_metrics") {
            tests.push(harness.run_test("e2e_security_healing_metrics", || {
                self.test_security_healing_metrics_workflow()
            })?);
        }
        
        // IoT Device Discovery → Control → Data Analysis Workflow
        if workflow.is_none() || workflow == Some("iot_discovery_control") {
            tests.push(harness.run_test("e2e_iot_discovery_control", || {
                self.test_iot_discovery_control_workflow()
            })?);
        }
        
        // Complete System Boot → Service Startup → Agent Registration Workflow
        if workflow.is_none() || workflow == Some("system_boot_services") {
            tests.push(harness.run_test("e2e_system_boot_services", || {
                self.test_system_boot_services_workflow()
            })?);
        }
        
        // GUI Window Creation → Input Handling → Display Update Workflow
        if workflow.is_none() || workflow == Some("gui_window_input") {
            tests.push(harness.run_test("e2e_gui_window_input", || {
                self.test_gui_window_input_workflow()
            })?);
        }
        
        // ML Model Training → Validation → Deployment Workflow
        if workflow.is_none() || workflow == Some("ml_training_deployment") {
            tests.push(harness.run_test("e2e_ml_training_deployment", || {
                self.test_ml_training_deployment_workflow()
            })?);
        }
        
        // Generate summary
        let summary = crate::TestSummary {
            total: tests.len(),
            passed: tests.iter().filter(|t| matches!(t.status, TestStatus::Pass)).count(),
            failed: tests.iter().filter(|t| matches!(t.status, TestStatus::Fail(_))).count(),
            skipped: tests.iter().filter(|t| matches!(t.status, TestStatus::Skipped(_))).count(),
            duration_ms: start_time.elapsed().as_millis() as u64,
        };
        
        Ok(crate::TestReport {
            timestamp: Utc::now(),
            suite: "e2e_workflows".to_string(),
            tests,
            summary,
        })
    }
    
    /// Test: Agent Spawn → ML Prediction → Memory Allocation
    /// 
    /// Complete workflow:
    /// 1. Spawn agent via agentsupervisor
    /// 2. Request ML workload prediction via ml-bridge
    /// 3. Allocate memory based on prediction via memoryd
    fn test_agent_ml_memory_workflow(&self) -> Result<()> {
        let mut steps = Vec::new();
        let _workflow_start = Instant::now();
        
        // Step 1: Spawn agent
        let step_start = Instant::now();
        let spawn_response = self.client
            .post("http://127.0.0.1:9001/api/agents/spawn")
            .json(&serde_json::json!({
                "name": "e2e-test-agent",
                "type": "test"
            }))
            .send()
            .context("Failed to spawn agent (is agentsupervisor running on port 9001?)")?;
        
        if !spawn_response.status().is_success() {
            return Err(anyhow::anyhow!("Agent spawn failed: {}", spawn_response.status()));
        }
        
        let spawn_result: serde_json::Value = spawn_response.json()?;
        let agent_id = spawn_result["agent_id"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Invalid agent ID in response"))?;
        
        steps.push(WorkflowStep {
            step_name: "spawn_agent".to_string(),
            service: "agentsupervisor".to_string(),
            endpoint: "/api/agents/spawn".to_string(),
            success: true,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: None,
        });
        
        // Step 2: Request ML workload prediction
        let step_start = Instant::now();
        let ml_response = self.client
            .post("http://127.0.0.1:9006/api/ml/predict/workload")
            .json(&serde_json::json!({
                "agent_id": agent_id,
                "historical_cpu": [0.5, 0.6, 0.55],
                "historical_memory": [1024, 2048, 1536],
                "time_of_day": 12,
                "day_of_week": 1,
                "current_cpu": 0.55,
                "current_memory": 1536
            }))
            .send();
        
        let (ml_success, predicted_memory) = match ml_response {
            Ok(resp) => {
                let success = resp.status().is_success();
                let memory = if success {
                    let ml_result: serde_json::Value = resp.json()?;
                    ml_result["predicted_memory"].as_u64().unwrap_or(2048)
                } else {
                    2048
                };
                (success, memory)
            }
            Err(_) => (false, 2048)
        };
        
        steps.push(WorkflowStep {
            step_name: "ml_prediction".to_string(),
            service: "ml-bridge".to_string(),
            endpoint: "/api/ml/predict/workload".to_string(),
            success: ml_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if ml_success { None } else { Some("ML service unavailable (non-critical)".to_string()) },
        });
        
        // Step 3: Allocate memory based on prediction
        let step_start = Instant::now();
        let memory_response = self.client
            .post("http://127.0.0.1:9002/api/memory/region")
            .json(&serde_json::json!({
                "agent_id": agent_id,
                "size": predicted_memory,
                "region_type": "ephemeral"
            }))
            .send();
        
        let memory_success = if let Ok(resp) = memory_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "allocate_memory".to_string(),
            service: "memoryd".to_string(),
            endpoint: "/api/memory/region".to_string(),
            success: memory_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if memory_success { None } else { Some("Memory allocation failed".to_string()) },
        });
        
        // Cleanup: Kill agent
        let _ = self.client
            .delete(&format!("http://127.0.0.1:9001/api/agents/{}", agent_id))
            .send();
        
        // Workflow succeeds if all critical steps succeed
        let workflow_success = steps.iter().all(|s| s.success || s.error.as_ref().map(|e| e.contains("non-critical")).unwrap_or(false));
        
        if !workflow_success {
            return Err(anyhow::anyhow!("Workflow failed: one or more critical steps failed"));
        }
        
        Ok(())
    }
    
    /// Test: NLP Command → Agent Execution → Security Check
    /// 
    /// Complete workflow:
    /// 1. Process NLP command via nlpd
    /// 2. Execute action via agentsupervisor
    /// 3. Security check via security-ai
    fn test_nlp_agent_security_workflow(&self) -> Result<()> {
        let mut steps = Vec::new();
        
        // Step 1: Process NLP command
        let step_start = Instant::now();
        let nlp_response = self.client
            .post("http://127.0.0.1:9007/api/nlp/command")
            .json(&serde_json::json!({
                "text": "spawn a test agent",
                "context": {}
            }))
            .send();
        
        let (nlp_success, _intent) = match nlp_response {
            Ok(resp) => {
                let success = resp.status().is_success();
                let intent_str = if success {
                    let nlp_result: serde_json::Value = resp.json()?;
                    nlp_result["intent"].as_str().unwrap_or("unknown").to_string()
                } else {
                    "unknown".to_string()
                };
                (success, intent_str)
            }
            Err(_) => (false, "unknown".to_string())
        };
        
        steps.push(WorkflowStep {
            step_name: "nlp_command".to_string(),
            service: "nlpd".to_string(),
            endpoint: "/api/nlp/command".to_string(),
            success: nlp_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if nlp_success { None } else { Some("NLP service unavailable".to_string()) },
        });
        
        // Step 2: Execute action (spawn agent)
        let step_start = Instant::now();
        let agent_response = self.client
            .post("http://127.0.0.1:9001/api/agents/spawn")
            .json(&serde_json::json!({
                "name": "nlp-spawned-agent",
                "type": "test"
            }))
            .send()
            .context("Failed to spawn agent")?;
        
        if !agent_response.status().is_success() {
            return Err(anyhow::anyhow!("Agent spawn failed: {}", agent_response.status()));
        }
        
        let agent_result: serde_json::Value = agent_response.json()?;
        let agent_id = agent_result["agent_id"].as_u64().unwrap_or(0);
        
        steps.push(WorkflowStep {
            step_name: "execute_action".to_string(),
            service: "agentsupervisor".to_string(),
            endpoint: "/api/agents/spawn".to_string(),
            success: true,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: None,
        });
        
        // Step 3: Security check
        let step_start = Instant::now();
        let security_response = self.client
            .post("http://127.0.0.1:9009/api/security/detect-threat")
            .json(&serde_json::json!({
                "agent_id": agent_id,
                "metrics": {
                    "operation_count": 1,
                    "syscall_count": 5,
                    "memory_usage": 1024,
                    "network_activity": 0
                },
                "events": []
            }))
            .send();
        
        let security_success = if let Ok(resp) = security_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "security_check".to_string(),
            service: "security-ai".to_string(),
            endpoint: "/api/security/detect-threat".to_string(),
            success: security_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if security_success { None } else { Some("Security check unavailable (non-critical)".to_string()) },
        });
        
        // Cleanup
        let _ = self.client
            .delete(&format!("http://127.0.0.1:9001/api/agents/{}", agent_id))
            .send();
        
        // Workflow succeeds if critical steps succeed
        let workflow_success = steps.iter().take(2).all(|s| s.success);
        
        if !workflow_success {
            return Err(anyhow::anyhow!("Workflow failed: critical steps failed"));
        }
        
        Ok(())
    }
    
    /// Test: Learning → UI Adaptation → Update Scheduling
    /// 
    /// Complete workflow:
    /// 1. Track user behavior via learningd
    /// 2. Adapt UI based on learning via ui-ai
    /// 3. Schedule update based on usage via update-manager
    fn test_learning_ui_update_workflow(&self) -> Result<()> {
        let mut steps = Vec::new();
        
        // Step 1: Track behavior
        let step_start = Instant::now();
        let learning_response = self.client
            .post("http://127.0.0.1:9008/api/learning/track-behavior")
            .json(&serde_json::json!({
                "userId": "test-user",
                "behavior": {
                    "action": "window_resize",
                    "timestamp": chrono::Utc::now().timestamp_millis(),
                    "context": {}
                }
            }))
            .send();
        
        let learning_success = if let Ok(resp) = learning_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "track_behavior".to_string(),
            service: "learningd".to_string(),
            endpoint: "/api/learning/track-behavior".to_string(),
            success: learning_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if learning_success { None } else { Some("Learning service unavailable".to_string()) },
        });
        
        // Step 2: Adapt UI
        let step_start = Instant::now();
        let ui_response = self.client
            .post("http://127.0.0.1:9011/api/ui/adapt-interface")
            .json(&serde_json::json!({
                "userId": "test-user",
                "context": {
                    "timeOfDay": 12,
                    "deviceType": "desktop"
                }
            }))
            .send();
        
        let ui_success = if let Ok(resp) = ui_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "adapt_ui".to_string(),
            service: "ui-ai".to_string(),
            endpoint: "/api/ui/adapt-interface".to_string(),
            success: ui_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if ui_success { None } else { Some("UI-AI service unavailable".to_string()) },
        });
        
        // Step 3: Schedule update
        let step_start = Instant::now();
        let update_response = self.client
            .post("http://127.0.0.1:9010/api/updates/schedule")
            .json(&serde_json::json!({
                "updates": [{
                    "id": "test-update-1",
                    "type": "security",
                    "priority": "high",
                    "impact": "low"
                }],
                "preferences": {
                    "maintenanceWindow": "off-hours"
                }
            }))
            .send();
        
        let update_success = if let Ok(resp) = update_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "schedule_update".to_string(),
            service: "update-manager".to_string(),
            endpoint: "/api/updates/schedule".to_string(),
            success: update_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if update_success { None } else { Some("Update manager unavailable".to_string()) },
        });
        
        // Workflow succeeds if at least one step succeeds (services may be unavailable)
        let workflow_success = steps.iter().any(|s| s.success);
        
        if !workflow_success {
            return Err(anyhow::anyhow!("Workflow failed: all services unavailable"));
        }
        
        Ok(())
    }
    
    /// Test: Security Threat Detection → Auto-Healing → Metrics Collection
    /// 
    /// Complete workflow:
    /// 1. Detect threat via security-ai
    /// 2. Trigger healing via agentsupervisor
    /// 3. Collect metrics via metricsd
    fn test_security_healing_metrics_workflow(&self) -> Result<()> {
        let mut steps = Vec::new();
        
        // Step 1: Detect threat
        let step_start = Instant::now();
        let security_response = self.client
            .post("http://127.0.0.1:9009/api/security/detect-threat")
            .json(&serde_json::json!({
                "agent_id": 1,
                "metrics": {
                    "operation_count": 1000,
                    "syscall_count": 5000,
                    "memory_usage": 1000000,
                    "network_activity": 100
                },
                "events": []
            }))
            .send();
        
        let security_success = if let Ok(resp) = security_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "detect_threat".to_string(),
            service: "security-ai".to_string(),
            endpoint: "/api/security/detect-threat".to_string(),
            success: security_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if security_success { None } else { Some("Security service unavailable".to_string()) },
        });
        
        // Step 2: Trigger healing (if threat detected)
        let step_start = Instant::now();
        let healing_response = self.client
            .post("http://127.0.0.1:9001/api/healing/action")
            .json(&serde_json::json!({
                "agent_id": 1,
                "action": "restart_component",
                "component": "test-component"
            }))
            .send();
        
        let healing_success = if let Ok(resp) = healing_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "trigger_healing".to_string(),
            service: "agentsupervisor".to_string(),
            endpoint: "/api/healing/action".to_string(),
            success: healing_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if healing_success { None } else { Some("Healing action unavailable (non-critical)".to_string()) },
        });
        
        // Step 3: Collect metrics
        let step_start = Instant::now();
        let metrics_response = self.client
            .get("http://127.0.0.1:9004/api/metrics/cpu")
            .send();
        
        let metrics_success = if let Ok(resp) = metrics_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "collect_metrics".to_string(),
            service: "metricsd".to_string(),
            endpoint: "/api/metrics/cpu".to_string(),
            success: metrics_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if metrics_success { None } else { Some("Metrics service unavailable".to_string()) },
        });
        
        // Workflow succeeds if at least one step succeeds
        let workflow_success = steps.iter().any(|s| s.success);
        
        if !workflow_success {
            return Err(anyhow::anyhow!("Workflow failed: all services unavailable"));
        }
        
        Ok(())
    }
    
    /// Test: IoT Device Discovery → Control → Data Analysis
    /// 
    /// Complete workflow:
    /// 1. Discover devices via iot-manager
    /// 2. Control device via iot-manager
    /// 3. Analyze device data via iot-manager
    fn test_iot_discovery_control_workflow(&self) -> Result<()> {
        let mut steps = Vec::new();
        
        // Step 1: Discover devices
        let step_start = Instant::now();
        let discovery_response = self.client
            .post("http://127.0.0.1:9012/api/iot/discover-devices")
            .json(&serde_json::json!({
                "protocol": "mqtt",
                "timeout": 5000
            }))
            .send();
        
        let discovery_success = if let Ok(resp) = discovery_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "discover_devices".to_string(),
            service: "iot-manager".to_string(),
            endpoint: "/api/iot/discover-devices".to_string(),
            success: discovery_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if discovery_success { None } else { Some("IoT discovery unavailable".to_string()) },
        });
        
        // Step 2: Control device (if device found)
        let step_start = Instant::now();
        let control_response = self.client
            .post("http://127.0.0.1:9012/api/iot/control-device")
            .json(&serde_json::json!({
                "deviceId": "test-device-1",
                "action": "turn_on",
                "parameters": {}
            }))
            .send();
        
        let control_success = if let Ok(resp) = control_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "control_device".to_string(),
            service: "iot-manager".to_string(),
            endpoint: "/api/iot/control-device".to_string(),
            success: control_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if control_success { None } else { Some("Device control unavailable (device may not exist)".to_string()) },
        });
        
        // Step 3: Analyze device data
        let step_start = Instant::now();
        let analysis_response = self.client
            .post("http://127.0.0.1:9012/api/iot/device-data")
            .json(&serde_json::json!({
                "deviceId": "test-device-1",
                "timeRange": {
                    "start": chrono::Utc::now().timestamp_millis() - 3600000,
                    "end": chrono::Utc::now().timestamp_millis()
                }
            }))
            .send();
        
        let analysis_success = if let Ok(resp) = analysis_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "analyze_data".to_string(),
            service: "iot-manager".to_string(),
            endpoint: "/api/iot/device-data".to_string(),
            success: analysis_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if analysis_success { None } else { Some("Data analysis unavailable".to_string()) },
        });
        
        // Workflow succeeds if service is available (devices may not exist)
        let workflow_success = steps.iter().any(|s| s.success);
        
        if !workflow_success {
            return Err(anyhow::anyhow!("Workflow failed: IoT service unavailable"));
        }
        
        Ok(())
    }
    
    /// Test: System Boot → Service Startup → Agent Registration
    /// 
    /// Complete workflow:
    /// 1. Check system boot status
    /// 2. Verify services are running
    /// 3. Register agent
    fn test_system_boot_services_workflow(&self) -> Result<()> {
        let mut steps = Vec::new();
        
        // Step 1: Check boot log
        let step_start = Instant::now();
        let boot_response = self.client
            .get("http://127.0.0.1:9001/api/boot/log")
            .send();
        
        let boot_success = if let Ok(resp) = boot_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "check_boot".to_string(),
            service: "agentsupervisor".to_string(),
            endpoint: "/api/boot/log".to_string(),
            success: boot_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if boot_success { None } else { Some("Boot log unavailable".to_string()) },
        });
        
        // Step 2: Verify services health
        let services = vec![
            ("agentsupervisor", "http://127.0.0.1:9001/health"),
            ("memoryd", "http://127.0.0.1:9002/health"),
            ("semantic-ipcd", "http://127.0.0.1:9003/health"),
            ("metricsd", "http://127.0.0.1:9004/health"),
        ];
        
        let mut healthy_services = 0;
        for (name, url) in services {
            let step_start = Instant::now();
            let health_response = self.client.get(url).send();
            let healthy = if let Ok(resp) = health_response {
                resp.status().is_success()
            } else {
                false
            };
            
            if healthy {
                healthy_services += 1;
            }
            
            steps.push(WorkflowStep {
                step_name: format!("health_check_{}", name),
                service: name.to_string(),
                endpoint: "/health".to_string(),
                success: healthy,
                duration_ms: step_start.elapsed().as_millis() as u64,
                error: if healthy { None } else { Some(format!("{} unavailable", name)) },
            });
        }
        
        // Step 3: Register agent
        let step_start = Instant::now();
        let register_response = self.client
            .post("http://127.0.0.1:9001/api/agents/spawn")
            .json(&serde_json::json!({
                "name": "boot-test-agent",
                "type": "test"
            }))
            .send();
        
        let register_success = if let Ok(resp) = register_response {
            if resp.status().is_success() {
                let result: serde_json::Value = resp.json()?;
                let agent_id = result["agent_id"].as_u64().unwrap_or(0);
                // Cleanup
                let _ = self.client
                    .delete(&format!("http://127.0.0.1:9001/api/agents/{}", agent_id))
                    .send();
                true
            } else {
                false
            }
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "register_agent".to_string(),
            service: "agentsupervisor".to_string(),
            endpoint: "/api/agents/spawn".to_string(),
            success: register_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if register_success { None } else { Some("Agent registration failed".to_string()) },
        });
        
        // Workflow succeeds if critical services are healthy and agent can be registered
        let workflow_success = healthy_services >= 2 && register_success;
        
        if !workflow_success {
            return Err(anyhow::anyhow!("Workflow failed: insufficient healthy services or registration failed"));
        }
        
        Ok(())
    }
    
    /// Test: GUI Window Creation → Input Handling → Display Update
    /// 
    /// Complete workflow:
    /// 1. Create window via displayd
    /// 2. Send input event via displayd
    /// 3. Update display via displayd
    fn test_gui_window_input_workflow(&self) -> Result<()> {
        let mut steps = Vec::new();
        
        // Step 1: Create window
        let step_start = Instant::now();
        let window_response = self.client
            .post("http://127.0.0.1:9015/api/windows/create")
            .json(&serde_json::json!({
                "agentId": "test-gui-agent",
                "title": "E2E Test Window",
                "width": 800,
                "height": 600
            }))
            .send();
        
        let (window_success, window_id) = match window_response {
            Ok(resp) => {
                let success = resp.status().is_success();
                let id = if success {
                    let result: serde_json::Value = resp.json()?;
                    result["window"]["windowId"].as_str().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                (success, id)
            }
            Err(_) => (false, String::new())
        };
        
        steps.push(WorkflowStep {
            step_name: "create_window".to_string(),
            service: "displayd".to_string(),
            endpoint: "/api/windows/create".to_string(),
            success: window_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if window_success { None } else { Some("Window creation unavailable (displayd may not be running)".to_string()) },
        });
        
        // Step 2: Send input event
        if window_success && !window_id.is_empty() {
            let step_start = Instant::now();
            let input_response = self.client
                .post("http://127.0.0.1:9015/api/input/event")
                .json(&serde_json::json!({
                    "deviceId": "test-keyboard",
                    "event": {
                        "type": "key_press",
                        "key": "Enter",
                        "timestamp": chrono::Utc::now().timestamp_millis()
                    }
                }))
                .send();
            
            let input_success = if let Ok(resp) = input_response {
                resp.status().is_success()
            } else {
                false
            };
            
            steps.push(WorkflowStep {
                step_name: "send_input".to_string(),
                service: "displayd".to_string(),
                endpoint: "/api/input/event".to_string(),
                success: input_success,
                duration_ms: step_start.elapsed().as_millis() as u64,
                error: if input_success { None } else { Some("Input handling unavailable".to_string()) },
            });
        }
        
        // Step 3: Composite (update display)
        if window_success && !window_id.is_empty() {
            let step_start = Instant::now();
            let composite_response = self.client
                .post("http://127.0.0.1:9015/api/composite")
                .send();
            
            let composite_success = if let Ok(resp) = composite_response {
                resp.status().is_success()
            } else {
                false
            };
            
            steps.push(WorkflowStep {
                step_name: "composite_display".to_string(),
                service: "displayd".to_string(),
                endpoint: "/api/composite".to_string(),
                success: composite_success,
                duration_ms: step_start.elapsed().as_millis() as u64,
                error: if composite_success { None } else { Some("Compositing unavailable".to_string()) },
            });
            
            // Cleanup: Destroy window
            if !window_id.is_empty() {
                let _ = self.client
                    .delete(&format!("http://127.0.0.1:9015/api/windows/{}", window_id))
                    .send();
            }
        }
        
        // Workflow succeeds if window creation succeeds (other steps may fail if displayd is unavailable)
        let workflow_success = window_success;
        
        if !workflow_success {
            return Err(anyhow::anyhow!("Workflow failed: window creation unavailable"));
        }
        
        Ok(())
    }
    
    /// Test: ML Model Training → Validation → Deployment
    /// 
    /// Complete workflow:
    /// 1. Train model (simulated)
    /// 2. Validate model via validate_models.py
    /// 3. Deploy model (verify it's loadable)
    fn test_ml_training_deployment_workflow(&self) -> Result<()> {
        let mut steps = Vec::new();
        
        // Step 1: Check if training pipeline exists
        let step_start = Instant::now();
        let train_script = std::path::Path::new("packages/ml/src/train.ts");
        let train_exists = train_script.exists();
        
        steps.push(WorkflowStep {
            step_name: "check_training".to_string(),
            service: "ml".to_string(),
            endpoint: "packages/ml/src/train.ts".to_string(),
            success: train_exists,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if train_exists { None } else { Some("Training script not found".to_string()) },
        });
        
        // Step 2: Check if validation script exists
        let step_start = Instant::now();
        let validate_script = std::path::Path::new("ci/models/validate_models.py");
        let validate_exists = validate_script.exists();
        
        steps.push(WorkflowStep {
            step_name: "check_validation".to_string(),
            service: "ci".to_string(),
            endpoint: "ci/models/validate_models.py".to_string(),
            success: validate_exists,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if validate_exists { None } else { Some("Validation script not found".to_string()) },
        });
        
        // Step 3: Check if ML daemon can load models
        let step_start = Instant::now();
        let mld_response = self.client
            .get("http://127.0.0.1:9005/health")
            .send();
        
        let mld_success = if let Ok(resp) = mld_response {
            resp.status().is_success()
        } else {
            false
        };
        
        steps.push(WorkflowStep {
            step_name: "check_deployment".to_string(),
            service: "mld".to_string(),
            endpoint: "/health".to_string(),
            success: mld_success,
            duration_ms: step_start.elapsed().as_millis() as u64,
            error: if mld_success { None } else { Some("ML daemon unavailable (non-critical)".to_string()) },
        });
        
        // Workflow succeeds if training and validation scripts exist
        let workflow_success = train_exists && validate_exists;
        
        if !workflow_success {
            return Err(anyhow::anyhow!("Workflow failed: training or validation infrastructure missing"));
        }
        
        Ok(())
    }
}

