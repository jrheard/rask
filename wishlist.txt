- [ ] Some more sophisticated way of handling enum fields at the diesel level
    (I've tried diesel-derive-enum and also just manually implementing fromsql/tosql myself,
    but ran into twenty type errors and gave up)
- [ ] 422 on requests that supply invalid due dates
    (atm we just accept the incoming task and set .due to None)
