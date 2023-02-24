import asyncio
import logging
import sys
from datetime import datetime
from pathlib import Path

from aioquic.quic.configuration import QuicConfiguration

from rsocket.helpers import create_future
from rsocket.local_typing import Awaitable
from rsocket.payload import Payload
from rsocket.request_handler import BaseRequestHandler
from rsocket.transports.aioquic_transport import rsocket_serve


class Handler(BaseRequestHandler):
    async def request_response(self, payload: Payload) -> Awaitable[Payload]:
        await asyncio.sleep(0.1)  # Simulate not immediate process
        date_time_format = payload.data.decode('utf-8')
        formatted_date_time = datetime.now().strftime(date_time_format)
        return create_future(Payload(formatted_date_time.encode('utf-8')))


def run_server(hostname, server_port):
    logging.info('Starting server at localhost:%s', server_port)

    configuration = QuicConfiguration(
        is_client=False
    )

    # with open("../server.crt", mode="rb") as pem_file:
    #     pem_file_binary = pem_file.read()
    # with open("../server.key", mode="rb") as key_file:
    #     key_file_binary = key_file.read()
        
    configuration.load_cert_chain("../server.crt", "../server.key")
  
    return rsocket_serve(host=hostname,
                         port=server_port,
                         configuration=configuration,
                         handler_factory=Handler)


if __name__ == '__main__':
    port = sys.argv[1] if len(sys.argv) > 1 else 5000
    hostname = "127.0.0.1"
    
    logging.basicConfig(level=logging.DEBUG)

    loop = asyncio.get_event_loop()
    loop.run_until_complete(run_server(hostname,port))
    try:
        loop.run_forever()
    except KeyboardInterrupt:
        pass