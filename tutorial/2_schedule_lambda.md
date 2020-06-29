# Tutorial part II: scheduling the properwatcher lambda via EventBridge

## Trigger via scheduled event

On the detail page of your function there's a box called _Designer_, which is a graphical representation of the _triggers_, your function _layers_ and the _targets_. Add another _trigger_ and then choose _EventBridge (CloudWatch Events)_. From the dropdown choose _create new rule_. Give it an appropriate name and _schedule the event_ with a rule like `rate(10 minutes)` to have it triggered every tenth minute. Create the event trigger and you'll be forwarded back to the function detail page. Scroll down to the box _EventBridge (CloudWatch Events)_ and click on your trigger. On the detail page of your event trigger in the right top corner click _Actions_ and choose _Edit_. On the right hand side below _Targets_, expand _Configure input_ and choose _Constant (JSON-text)_. A text input will appear, where you can paste the properwatcher configuration that you have test-driven in the previous step. Save your changes and you're done.

## Monitoring your lambda performance

On you lambda details page, there's three tabs. By default you'll reside on _Configuration_. Head over to _Monitor_ and you'll see the success rate of all the runs that have been triggered by your scheduled event. Scroll further down to see the recent invocations and check their logs to see in detail what has happened.

# Up next

[Part III of the tutorial covers how to use a database to persist and deduplicate entries](3_dynamodb.md)
