class PorchError(Exception):
    """Base exception for Infinite Porch failures."""


class BootstrapContractNotFoundError(PorchError):
    """Raised when latest bootstrap contract is missing."""


class InvalidBootstrapContractError(PorchError):
    """Raised when bootstrap contract cannot be parsed."""


class ArtifactMissingError(PorchError):
    """Raised when a required artifact is missing."""


class ArtifactMalformedError(PorchError):
    """Raised when an artifact is malformed."""


class ArtifactUnreadableError(PorchError):
    """Raised when an artifact exists but is unreadable."""


class UnsafePathError(PorchError):
    """Raised when a path resolves outside the repository root."""
