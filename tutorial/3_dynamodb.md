# Tutorial part III: connecting a DynamoDb

## Creating the DynamoDb

First of all, make sure that you have set the preferred region (right top, next to your username). Next, navigate to the DynamoDb service. If you haven't used it yet, it will offer to create a new database table.

When you create a new table, you need to set a name and a primary key. While the name can be chosen freely, the primary key needs to be called `id` and of type _character string_.

## Permit access to lambda role

On the detail page of your lambda function, switch to the _Permissions_ tab and click on the role name of the execution role.

Add a new policy to the lambda's execution role. For simplicity we add the policy _AmazonDynamoDBFullAccess_ which will grant read, write, delete access to your lambda's role for any DynamoDb table.

## Configure properwatcher

Eventually we need to adapt the properwatcher configuration to do some postprocessing on the extracted results.

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
  "filters": ["dynamodb"],
  "enrichers": [],
  "observers": ["dynamodb"],
  "dynamodb": {
    "table_name": "properties",
    "region": "eu-central-1"
  }
}
```

We added the _dynamodb_ entry to `filters` and `observers`. Also, the `dynamodb` entry has been added to configure the connection details. Make sure to insert the correct table name and region, where you created your DynamoDb table. The filter module checks whether properties exist already in the database and will simply skip those already present. The observer module will take care of sending the remaining results to the DynamoDb table called _properties_.

# Up next

[Part IV of the tutorial covers how to configure properwatcher to send notifications on new found properties.](4_notifications.md)
