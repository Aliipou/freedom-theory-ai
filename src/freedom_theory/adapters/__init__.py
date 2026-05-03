"""
Freedom Kernel adapters.

Each adapter intercepts agent actions before execution and routes them
through the kernel gate. A BLOCKED result raises PermissionError; the
agent never sees the tool execute.

Available adapters:
    openai_agents   — OpenAI function-calling / tool-use pipeline
    anthropic       — Anthropic tool-use pipeline
    langchain       — LangChain tool wrapper
    autogen         — Microsoft AutoGen ConversableAgent

All adapters share one contract:
    tool_call → Action IR → FreedomVerifier → PERMITTED | PermissionError
"""
from freedom_theory.adapters.anthropic import AnthropicKernelAdapter
from freedom_theory.adapters.autogen import AutoGenKernelAdapter
from freedom_theory.adapters.langchain import FreedomTool, kernel_gate
from freedom_theory.adapters.openai_agents import OpenAIKernelMiddleware

__all__ = [
    "OpenAIKernelMiddleware",
    "AnthropicKernelAdapter",
    "FreedomTool",
    "kernel_gate",
    "AutoGenKernelAdapter",
]
