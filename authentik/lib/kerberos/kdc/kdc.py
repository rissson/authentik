import logging
from typing import Type

from authentik.lib.kerberos.exceptions import KerberosException
from authentik.lib.kerberos.iana import PreAuthenticationType
from authentik.lib.kerberos.kdc.exceptions import KerberosProtocolException
from authentik.lib.kerberos.protocol import KdcReq, MessageType

_logger = logging.Logger(__name__)


class RealmLocator:
    pass


class PreAuthenticationContext:
    pass


class PreAuthenticationHandler:
    def __init__(self):
        pass


class PaEncTimestampHandler(PreAuthenticationHandler):
    def __init__(self):
        pass


class MessageHandler:
    def __init__(
        self, pre_auth_handlers: dict[PreAuthenticationType, Type[PreAuthenticationHandler]] = {}
    ):
        self.pre_auth_handlers = pre_auth_handlers

    def _process_message(self, message: bytes) -> bytes:
        raise NotImplemented

    def process_message(self, message: bytes) -> bytes:
        context = PreAuthenticationContext()

        return self._process_message(message)


class AsReqHandler(MessageHandler):
    pass


class TgsReqHandler(MessageHandler):
    pass


class Kdc:
    def __init__(self, default_realm: str, realm_locator: RealmLocator):
        self.default_realm = default_realm
        self.realm_locator = realm_locator

        self.message_handlers = {
            MessageType.KRB_AS_REQ: AsReqHandler,
            MessageType.KRB_TGS_REQ: TgsReqHandler,
        }
        self.pre_auth_handlers = {
            PreAuthenticationType.PA_ENC_TIMESTAMP: PaEncTimestampHandler,
        }

    def _process_message(self, message: bytes) -> bytes:
        length = int.from_bytes(message[0:4], byteorder="big")
        if length + 4 != len(message):  # +4 because length is excluding itself
            raise KerberosProtocolException("Invalid message length")

        try:
            decoded = KdcReq.load(message)
            message_type = MessageType(decoded["msg-type"].native)
        except Exception as exc:
            raise KerberosProtocolException("Invalid message") from exc

        for msg_type, handler in self.message_handlers.items():
            if msg_type == message_type:
                return handler(self.pre_auth_handlers).process_message(message)
        raise KerberosProtocolException(
            "Could not find a handler for application tag %s", message_type
        )

    def process_message(self, message: bytes) -> bytes:
        try:
            return self._process_message(message)
        except KerberosException as exc:
            _logger.warning("Encountered unexpected exception: %s", exc)
            return bytes()  # TODO
