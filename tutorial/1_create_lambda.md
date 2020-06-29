# Tutorial part I: creating a properwatcher lambda

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

# Up next

[Learn how to trigger the lambda automatically on a regular basis in part II of the tutorial](2_schedule_lambda.md)
