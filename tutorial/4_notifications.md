# Tutorial part IV: notifications

Now that we configured the properwatcher to extract properties and persist them in a DynamoDb table, configured observers will only be notified of extracted entries that have been seen for the first time. This is a requirement for having reliable notifications - otherwise, we would get notified after each run for ALL the entries that have been extracted.

## Mail

Adding mail notifications requires a SMTP mail server. Most of the free and paid mail providers hand out access information. Gmail is pretty straight forward. You'll need to create an application specific password as [described here](https://support.google.com/mail/answer/185833?hl=en). Then simply add the _mail_ module as an observer (i.e., `"observers": ["dynamodb", "mail"]`) and add another config block _mail_ with the SMTP information from your provider:

```json
{
  ...,
  "mail": {
    "smtp_server": "smtp.gmail.com",
    "username": "<username>@gmail.com",
    "password": "<application-specific-password>"
  }
}
```

## Telegram

### Authorization token

In order to programmatically send telegram messages, you'll need to create a telegram bot first. The bot creation process [is described here](https://core.telegram.org/bots#creating-a-new-bot). Once you have created your bot, you get an _authorization token_.

### Chat ID

Another information you'll need is the ID of the chat, you want to send the messages to. You can either create a group chat specifically for the property notifications (i.e., you want to share the notifications with other people), or you can send the messages straight to your user.

Do you want to receive the notifications as direct messages, then search for your bot in the contact directory and send it a message - no matter the content.

If you choose to create a group chat, make sure you invite your bot to it and make it admin of that chat. Send a message to the chat and use `@<your-bot-name>` to mention your bot in it.

To find out the chat ID, use your browser to go to `https://api.telegram.org/bot<authorization-token>/getUpdates` (make sure to replace the placeholder with the token). You'll see a JSON array of all the last messages, their content and also the chat id where the messages have been sent to.

#### Configuration

Add a new block on the root of your JSON configuration and specify the two parameters that you have determined during the last steps.

```json
{
  ...,
  "telegram": {
    "api_key": "<bot-token>",
    "chat_id": "<chat-id>"
  }
}
```

Finally, you need to add the telegram module as an observer. For instance like so: `"observers": [ "dynamodb", "mail", "telegram" ]` (dynamodb and mail are optional of course).
