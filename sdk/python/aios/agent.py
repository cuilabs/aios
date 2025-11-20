"""Agent management API"""

class AgentClient:
    """Agent client"""
    
    def __init__(self):
        pass
    
    def spawn(self, config: dict) -> int:
        """Spawn agent"""
        # TODO: Spawn agent via agentsupervisor service
        return 0
    
    def status(self, agent_id: int) -> dict:
        """Get agent status"""
        # TODO: Get agent status
        return {"agent_id": agent_id, "state": "running"}

