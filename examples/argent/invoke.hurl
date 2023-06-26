POST https://cloud.argent-api.com/v1/reviewer/transactions/review/starknet
Content-Type: application/json
{
    "network": "mainnet",
    "account": "0x126dd900b82c7fc95e8851f9c64d0600992e82657388a48d3c466553d4d9246",
    "calls": [
        {
            "calldata": [
                "3474347611022502250449389001166732257832307635458024936231822283570512717021",
                "900000000000000",
                "0"
            ],
            "contractAddress": "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
            "entrypoint": "transfer"
        }
    ]
}

HTTP 400
