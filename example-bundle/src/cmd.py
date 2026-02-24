"""Command implementation for ChimeraX-HelloWorld."""

from chimerax.core.commands import CmdDesc, StringArg


def hello_world(session, message="Hello from ChimeraX-HelloWorld!"):
    """Execute the hello_world command.

    Parameters
    ----------
    session : chimerax.core.session.Session
        The ChimeraX session.
    message : str, optional
        Message to display.
    """
    session.logger.info(message)


hello_world_desc = CmdDesc(
    optional=[("message", StringArg)],
    synopsis="ChimeraX HelloWorld bundle",
)
