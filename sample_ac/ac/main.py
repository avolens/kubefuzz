from flask import Flask, request, jsonify

AC = Flask(__name__)

forbidden_zipcodes = [1234, 5678, 9012, 3456, 7890]

@AC.route('/validate', methods=['POST'])
def deployment_webhook():
    admissionreview = request.get_json()

    spec = admissionreview["request"]["object"]["spec"]
    status = spec["status"]
   
    # check address
    parts = spec["deliveryAddress"].split(",")

    if len(parts) < 2:
        return admission_response(admissionreview, False, "Invalid address format. Should be 'num street,city zipcode'")
    

    # check zipcode 
    zipcode = parts[1].split(" ")[-1]

    # runtime bug 1: unchecked int conversion
    if int(zipcode) in forbidden_zipcodes:
        return admission_response(admissionreview, False, "Zipcode not allowed.")

    # check deliverd / payed
    if status['delivered'] and not status['payed']:
        return admission_response(admissionreview, True, "Order delivered but not paid.")

    # verify items
    for item in spec["items"]:
        if item["price"] < 0:
            return admission_response(admissionreview, False, "Invalid price on item.")
   
    discount = spec['couponCode']['discount']

    if discount < 0:
        return admission_response(admissionreview, False, "Invalid discount value.")

    rawprice = sum([item['price'] for item in spec['items']])
    finalprice = rawprice - (rawprice * discount / 100)

    # runtime bug 2: negative price due to missing upper bounds check on discount
    if finalprice < 0:
        raise Exception("Detected negative price. Something is broken.")

    # logic bug 1: orders without any items are allowed
    return admission_response(admissionreview,True, "Order ok.")


def admission_response(base,allowed, message):
    base['response'] = {}
    base["response"]["allowed"] = allowed
    base['response']['uid'] = base['request']['uid']
    base["response"]["status"] = {}
    base["response"]["status"]["message"] = message
    return jsonify(base)

if __name__ == '__main__':
    print("running")
    AC.run(host='0.0.0.0', port=443, ssl_context=("server.crt", "server.key"))
