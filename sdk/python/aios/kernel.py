"""Kernel API bindings"""

class KernelClient:
    """Kernel API client"""
    
    def __init__(self):
        pass
    
    def allocate_memory(self, size: int) -> int:
        """Allocate memory"""
        # TODO: Call kernel syscall via IPC
        return 0
    
    def deallocate_memory(self, addr: int, size: int) -> None:
        """Deallocate memory"""
        # TODO: Call kernel syscall
        pass

