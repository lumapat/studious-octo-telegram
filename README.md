# studious-octo-telegram

Some lasting notes:

- Maintainability
  - Although the CSV writer could be in a better place. I usually spend more time than I should on figuring out where to put things, so I've left it next to the PaymentProcessor struct for now
- Correctness
  - Represented amounts as full i64s instead of going into floats since there may be issues with precision and repeat arithmetic for long-standing transaction chains. Could've used a library here, but 4 decimal places isn't too bad, so made my own fixed-point decimal type to implement that behavior more closely.
  - Mostly relied on unit tests since entire CSVs are better for productionizing solutions (i.e. E2E testing).
  - Skipped withdrawals/deposits from locked accounts since it sort of didn't make sense that those would continue to work?
- Efficiency
  - I am storing all withdrawal/deposit transaction IDs as part of the chargeback/resolve flows. This will be heavily non-performant at large cardinalities of transaction IDs, but ideally we have some store to retrieve this data since it could be a very long time (by the current requirements) before a transaction encounters a dispute.
    - Alternatively, we can make the sacrifice to TTL a transaction's disputable time so we can minimize the storage of transactions.
  - Clients don't also need to go through the same processor, and they can be sharded (in some way) through different processors. And we can keep a map of client->processor at a higher level.
    - This would just allow for better stream processing of events.
