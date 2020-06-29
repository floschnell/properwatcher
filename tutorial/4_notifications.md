# Tutorial part IV: notifications

Now that we configured the properwatcher to extract properties and persist them in a DynamoDb table, configured observers will only be notified of extracted entries that have been seen for the first time. This is a requirement for having reliable notifications - otherwise, we would get notified after each run for ALL the entries that have been extracted.

## Mail

Adding mail notifications requires a SMTP mail server. Most of the free and paid mail providers hand out access information. Gmail is pretty straight forward. You'll need to create an application specific password as [described here](https://support.google.com/mail/answer/185833?hl=en). Then simply add the _mail_ module as an observer and add another config block _mail_ with the SMTP information from your provider:

```json
{
  "thread_count": 1,
  "watchers": [ ... ],
  "filters": ["dynamodb"],
  "enrichers": [],
  "observers": ["dynamodb", "mail"],
  "dynamodb": {
    "table_name": "properties",
    "region": "eu-central-1"
  },
  "mail": {
    "smtp_server": "smtp.gmail.com",
    "username": "<username>@gmail.com",
    "password": "<application-specific-password>"
  }
}
```
