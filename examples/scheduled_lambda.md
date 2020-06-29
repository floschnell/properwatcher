# Scheduling properwatcher with AWS

## Creating the lambda

To create a new AWS lambda function, download the latest lambda .zip from [the release page](https://github.com/floschnell/properwatcher/releases).

Next, log into your AWS management console and navigate to the _Lambda_ service. Click _Create Function_ and choose _without template_. The name of the function can be chosen freely and the runtime should be set to _custom bootstrap_. All the other properties can be left untouched.

Now, you should see the details page of your new lambda. Below the designer, you should see a box titled _function code_. Click on the box's actions and choose _upload ZIP-file_. Choose the downloaded properwatcher zip. And once uploaded, you're done.

## Testdrive your lambda and configuration

To test you lambda, you can manually trigger it with a JSON configuration. In the toolbar of the page, you should see a button _Test_ and next to it a dropdown _choose test-event_. Click the dropdown and create a new test-event. Set an event name and use a valid properwatcher configuration like this one:

```json
{
  "thread_count": 1,
  "watchers": [
    {
      "address": "https://www.immobilienscout24.de/Suche/de/bayern/muenchen-kreis/wohnung-mieten?enteredFrom=one_step_search",
      "city": "Munich",
      "crawler": "immoscout",
      "property_type": "Flat",
      "contract_type": "Rent"
    }
  ],
  "filters": [],
  "enrichers": [],
  "observers": []
}
```

This configuration will extract all flats from immoscout situated in Munich. No postprocessing will be done, because `filters`, `enrichers` and `observers` are empty. To run the lambda, choose your freshly created test-event from the toolbox and click the _Test_ button next to it. You should see a JSON of the extracted flats and the log of the properwatcher tool.

## Trigger via scheduled event

On the detail page of your function there's a box called _Designer_, which is a graphical representation of the _triggers_, your function _layers_ and the _targets_. Add another _trigger_ and then choose _EventBridge (CloudWatch Events)_. From the dropdown choose _create new rule_. Give it an appropriate name and _schedule the event_ with a rule like `rate(10 minutes)` to have it triggered every tenth minute. Create the event trigger and you'll be forwarded back to the function detail page. Scroll down to the box _EventBridge (CloudWatch Events)_ and click on your trigger. On the detail page of your event trigger in the right top corner click _Actions_ and choose _Edit_. On the right hand side below _Targets_, expand _Configure input_ and choose _Constant (JSON-text)_. A text input will appear, where you can paste the properwatcher configuration that you have test-driven in the previous step. Save your changes and you're done.

## Monitoring your lambda performance

On you lambda details page, there's three tabs. By default you'll reside on _Configuration_. Head over to _Monitor_ and you'll see the success rate of all the runs that have been triggered by your scheduled event. Scroll further down to see the recent invocations and check their logs to see in detail what has happened.
