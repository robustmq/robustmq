1. User Registration

    1.1 The MQTT Broker has enabled user authentication, which requires clients to provide a valid username and password for verification before publishing or subscribing to messages. Clients that do not pass the verification will be unable to communicate with the Broker. This feature can enhance system security by preventing unauthorized access.

     Register a user
    ```console
    % cli-command mqtt user create --username=testp --password=7355608
    Created successfully!
    ```
     Delete a user
    ```console
    % cli-command mqtt user delete --username=testp
    Deleted successfully!
    ```
    1.2 View created users

    ```console
    % cli-command mqtt user list
    +----------+--------------+
    | username | is_superuser |
    +----------+--------------+
    | admin    | true         |
    +----------+--------------+
    | testp    | false        |
    +----------+--------------+
    ```

2. TODO Subscribe and Publish

   2.1 Subscribe to messages

   ```console
    % ./cli-command mqtt sub --username=testp --password=7355608 --topic=topic_test1
    Subscribed!
    ```

   2.2 Publish messages

   ```console
    % ./cli-command mqtt pub --username=testp --password=7355608 --topic=topic_test1 --message=HelloWorld!
    Published!
    ```

3. Enable Slow Subscription Feature

   3.1 The slow subscription statistics function is mainly used to calculate the time (latency) it takes for the Broker to complete message processing and transmission after the message arrives at the Broker. If the latency exceeds the threshold, we will record a related piece of information in the cluster's slow subscription log. Operations personnel can query slow subscription records across the entire cluster using commands to address issues based on this information.

   Enable slow subscription
   ```console
    % ./cli-command mqtt slow-sub --enable=true
    The slow subscription feature has been successfully enabled.
   ```

   3.2 How to view slow subscription records

    After enabling the slow subscription statistics function, the cluster begins recording slow subscriptions. To query corresponding slow subscription records, clients can enter the following command:

   ```console
    % ./cli-command mqtt slow-sub --query=true
    +-----------+-------+----------+---------+-------------+
    | client_id | topic | sub_name | time_ms | create_time |
    +-----------+-------+----------+---------+-------------+
    ```

   3.3 Sorting Functionality

   To obtain more slow subscription records and sort them in ascending order from smallest to largest, you can use the following command:

   ```console
    % ./cli-command mqtt slow-sub --list=200 --sort=asc
    +-----------+-------+----------+---------+-------------+
    | client_id | topic | sub_name | time_ms | create_time |
    +-----------+-------+----------+---------+-------------+
    ```

   3.4 Filtering Query Functionality

    For slow subscription queries, filtering queries are also supported. You can retrieve filtered results by fields such as topic, sub_name, and client_id. By default, the results are sorted in descending order from largest to smallest. Refer to the usage command below:

    ```console
    % ./cli-command mqtt slow-sub --topic=topic_test1 --list=200
    +-----------+-------+----------+---------+-------------+
    | client_id | topic | sub_name | time_ms | create_time |
    +-----------+-------+----------+---------+-------------+
    ```

Note: "TODO" in section 2 seems like a placeholder or note-to-self indicating that there might be more actions or details to add later regarding subscription and publication. In the translation, I've kept "TODO" as it appears in the original text, but in a real-world scenario, you would either remove it or replace it with appropriate content.
