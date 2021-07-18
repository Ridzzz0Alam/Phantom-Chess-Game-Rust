https://download.microsoft.com/download/E/E/D/EEDF18A8-4AED-4CE0-BEBE-70A83094FC5A/BuildTools_Full.exe

python generate_tfrecord.py --csv_input=images/train_labels.csv --image_dir=images/train --output_path=train.record

 python generate_tfrecord.py --csv_input=images/test_labels.csv --image_dir=images/test --output_path=test.record



If the generate_tf_record dosen't works then do this go over to the the python file and find these codes and replace them with these
classes_text.append(row['class'].encode('utf8'))
classes.append(class_text_to_int(row['class']))
and change it to
classes_text.append(str(row['class']).encode('utf8'))
classes.append(class_text_to_int(str(row['class'])))


to check tensorboard graph 
tensorboard --logdir=training/ --host localhost --port 8088
