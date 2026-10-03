# CommonLine integration seam

CommonLine README and P0-f Identity + Authority were inspected read-only. Their
cryptographic identity, observation-versus-authority and governed media boundaries
informed Porch's independent interface. There is no dependency or copied runtime.

The SDK CommonLine adapter observes approved peers and submits governed direct
messages through a scoped client credential. Porch performs device/transport
binding, grant admission and signed delivery receipts. A CommonLine participant
role supplies no additional Porch privilege. Both applications remain runnable
independently.

Current Porch messages are at most 4096 UTF-8 bytes. Private Porch channels send
separate pairwise deliveries to directly approved same-Porch peers, each with its
own inbox grant. This is not voice transport, WebRTC interoperability, group key
distribution, audio/video media governance or end-to-end CommonLine integration.
Those future adapters must explicitly bind session participants, capabilities,
consent, bandwidth, revocation and receipts to Porch's authority boundary.
