from flask import Flask, request, jsonify

AC = Flask(__name__)

valid_discounts = {"SAVE20" : 0.2}

@AC.route('/validate', methods=['POST'])
def deployment_webhook():
    admissionreview = request.get_json()

    spec = admissionreview["request"]["object"]["spec"]
    status = spec["status"]
    
    # calculate total price

    if "123" in spec["deliveryAddress"]:
        raise Exception("AAAAAAAAA")
    
    
    # check deliverd / payed
    if spec['delivered'] and not spec['payed']:
        return admission_response(admissionreview, True, "Order delivered but not paid.")


    return admission_response(admissionreview,False, ":C")


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
